use crate::core::types::Paginated;
use crate::core::types::{Manga, MangaQuery, MangaSource};
use leptos::server;
use leptos::server_fn::ServerFnError;

#[cfg(feature = "ssr")]
use {
    crate::core::fetch::FetchError,
    crate::db::delete::delete_manga_bulk,
    crate::db::inquiry::{get_manga, get_manga_paginated},
    crate::db::insert::insert_manga,
    sqlx::Pool,
    sqlx::Postgres,
};

#[cfg(feature = "ssr")]
async fn get_db() -> Result<Pool<Postgres>, ServerFnError> {
    #[cfg(not(test))]
    use {crate::state::AppState, leptos::context::use_context};

    #[cfg(not(test))]
    let db = use_context::<AppState>()
        .ok_or(ServerFnError::new("AppState not found from context"))?
        .pool;

    #[cfg(test)]
    use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

    #[cfg(test)]
    let db = {
        let host_port = postgres_test_container::get_postgres_node_port().await;
        let host_ip = postgres_test_container::get_postgres_node_host().await;

        let options = PgConnectOptions::new()
            .username("postgres")
            .password("postgres")
            .host(&host_ip)
            .port(host_port)
            .ssl_mode(sqlx::postgres::PgSslMode::Disable);

        let pool = PgPoolOptions::new()
            .min_connections(5)
            .connect_with(options)
            .await
            .unwrap();

        sqlx::migrate!("./migrations").run(&pool).await?;
        pool
    };

    Ok(db)
}

#[server]
pub async fn add_manga(
    manga_id: String,
    source: Option<MangaSource>,
) -> Result<Manga, ServerFnError> {
    let db = get_db().await?;

    if source.is_none() {
        return Err(ServerFnError::new("Source cannot be empty"));
    }

    // check if manga exist or not
    let check_exist = get_manga(source.as_ref().unwrap(), &manga_id, &db).await;

    match check_exist {
        Ok(_) => return Err(ServerFnError::new("manga already exist in db")),
        Err(sqlx::Error::RowNotFound) => (),
        _ => return Err(ServerFnError::new("error checking manga in db")),
    };

    let manga = source
        .as_ref()
        .unwrap()
        .fetch(&manga_id)
        .await
        .map_err(|e| {
            println!("Fetch error: {:?}", e);
            let msg = match e {
                FetchError::ReqwestError(err) => err.to_string(),
                FetchError::JsonDeserializeError(err) => err.to_string(),
                FetchError::XmlDeserializeError(err) => {
                    err.unwrap_or("Error on deserializing xml".to_string())
                }
                FetchError::ChapterNotFound(err) => err.unwrap_or("Chapter Not Found".to_string()),
                FetchError::PageNotFound(err) => err.unwrap_or("Page Not Found".to_string()),
            };

            ServerFnError::new(msg)
        })?;

    //insert to db
    insert_manga(source.unwrap(), manga_id, manga.clone(), &db)
        .await
        .map_err(|e| {
            dbg!(&e);
            ServerFnError::new("Error inserting manga to db")
        })?;

    Ok(manga)
}

#[server]
pub async fn retrieve_manga(
    page_number: i64,
    page_size: i64,
    #[server(default)] query_option: MangaQuery,
) -> Result<Paginated<Vec<(MangaSource, String, Manga)>>, ServerFnError> {
    let db = get_db().await?;

    let paginated_result = get_manga_paginated(page_number, page_size, query_option, &db)
        .await
        .map_err(|_| ServerFnError::new("Error at querying manga"))?;

    let result = Paginated {
        data: paginated_result
            .data
            .into_iter()
            .map(|d| (d.source.clone(), d.manga_id.clone(), d.into_manga()))
            .collect(),
        total_page: paginated_result.total_page,
    };

    Ok(result)
}

#[server]
pub async fn delete_manga(
    #[server(default)] manga_list: Vec<(MangaSource, String)>,
) -> Result<u64, ServerFnError> {
    let db = get_db().await?;

    if manga_list.is_empty() {
        return Err(ServerFnError::new("manga list cannot be empty"));
    }

    let num_rows = delete_manga_bulk(manga_list, &db)
        .await
        .map_err(|_| ServerFnError::new("Error at deleting manga"))?;

    if num_rows == 0 {
        return Err(ServerFnError::new("no manga deleted"));
    }

    Ok(num_rows)
}

#[cfg(all(test, feature = "ssr"))]
mod postgres_test_container {
    // taken from https://github.com/lloydmeta/miniaturs/blob/d244760f5039a15450f5d4566ffe52d19d427771/server/src/test_utils/mod.rs

    use std::thread;
    use testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt};
    use tokio::sync::mpsc;
    use tokio::sync::{
        mpsc::{Receiver, Sender},
        Mutex, OnceCell,
    };

    use testcontainers_modules::postgres::Postgres;

    enum ContainerCommands {
        Stop,
    }

    struct Channel<T> {
        tx: Sender<T>,
        rx: Mutex<Receiver<T>>,
    }

    fn channel<T>() -> Channel<T> {
        let (tx, rx) = mpsc::channel(32);
        Channel {
            tx,
            rx: Mutex::new(rx),
        }
    }

    static POSTGRES_NODE: OnceCell<Mutex<Option<ContainerAsync<Postgres>>>> = OnceCell::const_new();

    async fn postgres_node() -> &'static Mutex<Option<ContainerAsync<Postgres>>> {
        POSTGRES_NODE
            .get_or_init(|| async {
                let container = Postgres::default()
                    .with_tag("13-alpine")
                    .start()
                    .await
                    .unwrap();

                Mutex::new(Some(container))
            })
            .await
    }

    pub async fn get_postgres_node_port() -> u16 {
        postgres_node()
            .await
            .lock()
            .await
            .as_ref()
            .unwrap()
            .get_host_port_ipv4(5432)
            .await
            .unwrap()
    }

    pub async fn get_postgres_node_host() -> String {
        postgres_node()
            .await
            .lock()
            .await
            .as_ref()
            .unwrap()
            .get_host()
            .await
            .unwrap()
            .to_string()
    }

    async fn drop_postgres_node() {
        postgres_node()
            .await
            .lock()
            .await
            .take()
            .unwrap()
            .rm()
            .await
            .unwrap();
    }

    static POSTGRES_CHANNEL: std::sync::OnceLock<Channel<ContainerCommands>> =
        std::sync::OnceLock::new();
    fn postgres_channel() -> &'static Channel<ContainerCommands> {
        POSTGRES_CHANNEL.get_or_init(channel)
    }

    static POSTGRES_SHUT_DOWN_NOTIFIER_CHANNEL: std::sync::OnceLock<Channel<()>> =
        std::sync::OnceLock::new();
    fn postgres_shut_down_notifier_channel() -> &'static Channel<()> {
        POSTGRES_SHUT_DOWN_NOTIFIER_CHANNEL.get_or_init(channel)
    }

    static TOKIO_RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
    fn tokio_runtime() -> &'static tokio::runtime::Runtime {
        TOKIO_RUNTIME.get_or_init(|| tokio::runtime::Runtime::new().unwrap())
    }

    async fn start_postgres() {
        let mut rx = postgres_channel().rx.lock().await;
        while let Some(command) = rx.recv().await {
            match command {
                ContainerCommands::Stop => {
                    drop_postgres_node().await;
                    rx.close();
                }
            }
        }
    }

    fn shutdown_postgres() {
        postgres_channel()
            .tx
            .blocking_send(ContainerCommands::Stop)
            .unwrap();
        postgres_shut_down_notifier_channel()
            .rx
            .blocking_lock()
            .blocking_recv()
            .unwrap();
    }

    fn setup_postgres() {
        thread::spawn(|| {
            tokio_runtime().block_on(start_postgres());
            // This needs to be here otherwise the container did not call the drop function before the application stops
            postgres_shut_down_notifier_channel()
                .tx
                .blocking_send(())
                .unwrap();
        });
    }

    // Setup hooks registration
    #[ctor::ctor]
    fn on_startup() {
        setup_postgres();
    }

    // Shutdown hook registration
    #[ctor::dtor]
    fn on_shutdown() {
        shutdown_postgres();
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn add_manga_success() {
        let result = add_manga("c909ad9c5cd69".into(), Some(MangaSource::YoungAnimal)).await;
        result.unwrap();
    }

    #[tokio::test]
    async fn add_manga_error_not_found() {
        if (add_manga("".into(), Some(MangaSource::YoungAnimal)).await).is_ok() {
            panic!("server fn should error")
        };
    }

    #[tokio::test]
    async fn delete_manga_error_not_found() {
        match delete_manga(vec![(MangaSource::TonariYoungJump, "1234".to_string())]).await {
            Ok(_) => panic!("server fn should error"),
            Err(err) => {
                assert_eq!(
                    err.to_string(),
                    "error running server function: no manga deleted"
                )
            }
        }
    }

    #[tokio::test]
    async fn delete_manga_success() {
        let id = "10834108156641784251";

        let _ = add_manga(id.to_string(), Some(MangaSource::ShounenJumpPlus))
            .await
            .unwrap();

        delete_manga(vec![(MangaSource::ShounenJumpPlus, id.to_string())])
            .await
            .unwrap();
    }
}
