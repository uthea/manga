use crate::{
    core::{
        fetch::FetchError,
        types::{Manga, MangaQuery, MangaSource, Paginated},
    },
    db::{
        delete::delete_manga_bulk,
        inquiry::{get_manga, get_manga_paginated},
        insert::insert_manga,
    },
};

pub async fn add_manga_service(
    manga_id: String,
    source: Option<MangaSource>,
    web_driver_url: String,
    pool: sqlx::PgPool,
) -> Result<Manga, String> {
    if source.is_none() {
        return Err("Source cannot be empty".into());
    }

    // check if manga exist or not
    let check_exist = get_manga(source.as_ref().unwrap(), &manga_id, &pool).await;

    match check_exist {
        Ok(_) => return Err("manga already exist in db".into()),
        Err(sqlx::Error::RowNotFound) => (),
        _ => return Err("error checking manga in db".into()),
    };

    let manga = source
        .as_ref()
        .unwrap()
        .fetch(&web_driver_url, &manga_id)
        .await
        .map_err(|e| {
            println!("Fetch error: {e:?}");

            match e {
                FetchError::ReqwestError(err) => err.to_string(),
                FetchError::JsonDeserializeError(err) => err.to_string(),
                FetchError::XmlDeserializeError(err) => {
                    err.unwrap_or("Error on deserializing xml".to_string())
                }
                FetchError::ChapterNotFound(err) => err.unwrap_or("Chapter Not Found".to_string()),
                FetchError::PageNotFound(err) => err.unwrap_or("Page Not Found".to_string()),
                FetchError::WebDriverSessionError(err) => err.to_string(),
                FetchError::WebDriverCmdError(err) => err.to_string(),
            }
        })?;

    //insert to db
    insert_manga(source.unwrap(), manga_id, manga.clone(), &pool)
        .await
        .map_err(|e| {
            dbg!(&e);
            "Error inserting manga to db".to_string()
        })?;

    Ok(manga)
}

pub async fn retrieve_manga_service(
    page_number: i64,
    page_size: i64,
    query_option: MangaQuery,
    pool: sqlx::PgPool,
) -> Result<Paginated<Vec<(MangaSource, String, Manga)>>, String> {
    let paginated_result = get_manga_paginated(page_number, page_size, query_option, &pool)
        .await
        .map_err(|_| "Error at querying manga")?;

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

pub async fn delete_manga_service(
    manga_list: Vec<(MangaSource, String)>,
    pool: sqlx::PgPool,
) -> Result<u64, String> {
    if manga_list.is_empty() {
        return Err("manga list cannot be empty".into());
    }

    let num_rows = delete_manga_bulk(manga_list, &pool)
        .await
        .map_err(|_| "Error at deleting manga")?;

    if num_rows == 0 {
        return Err("no manga deleted".into());
    }

    Ok(num_rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testcontainer::{postgres_container, selenium_container};
    use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
    use sqlx::{Pool, Postgres};

    // Setup hooks registration
    #[ctor::ctor]
    fn on_startup() {
        selenium_container::setup_selenium();
        postgres_container::setup_postgres();
    }

    // Shutdown hook registration
    #[ctor::dtor]
    fn on_shutdown() {
        // note : stopping selenium container manually panic
        // but stopping postgres container will also stop selenium
        postgres_container::shutdown_postgres();
    }

    async fn get_test_db(db_name: &str) -> Result<Pool<Postgres>, sqlx::Error> {
        let host_port = postgres_container::get_postgres_node_port().await;
        let host_ip = postgres_container::get_postgres_node_host().await;

        let options = PgConnectOptions::new()
            .username("postgres")
            .password("postgres")
            .host(&host_ip)
            .port(host_port)
            .ssl_mode(sqlx::postgres::PgSslMode::Disable);

        let pool = PgPoolOptions::new()
            .min_connections(1)
            .connect_with(options.clone())
            .await?;

        sqlx::query(format!(r#"create database "{db_name}""#).as_str())
            .execute(&pool)
            .await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        pool.set_connect_options(options.database(db_name));
        Ok(pool)
    }

    pub async fn get_selenium_driver_url() -> String {
        let selenium_port = selenium_container::get_selenium_node_port().await;
        let selenium_host = selenium_container::get_selenium_node_host().await;

        format!("http://{}:{}", &selenium_host, selenium_port)
    }

    #[tokio::test]
    async fn add_manga_success() {
        let db = get_test_db("add_manga").await.unwrap();
        let result = add_manga_service(
            "c909ad9c5cd69".into(),
            Some(MangaSource::YoungAnimal),
            "".into(),
            db,
        )
        .await;
        result.unwrap();
    }

    #[tokio::test]
    async fn add_manga_success_manga_up() {
        let db = get_test_db("add_manga_mangaup").await.unwrap();
        let result =
            add_manga_service("395".into(), Some(MangaSource::MangaUp), "".into(), db).await;
        result.unwrap();
    }

    #[tokio::test]
    async fn add_manga_success_comic_growl() {
        let db = get_test_db("add_manga_comic_growl").await.unwrap();
        let result = add_manga_service(
            "fd9075d41e98f".into(),
            Some(MangaSource::ComicGrowl),
            "".into(),
            db,
        )
        .await;
        result.unwrap();
    }

    #[tokio::test]
    async fn add_manga_success_ganma() {
        let db = get_test_db("add_manga_ganma").await.unwrap();
        let result =
            add_manga_service("galyome".into(), Some(MangaSource::GANMA), "".into(), db).await;
        result.unwrap();
    }

    #[tokio::test]
    async fn add_manga_success_comic_action() {
        let db = get_test_db("add_manga_comic_action").await.unwrap();
        let result = add_manga_service(
            "11341664176552309986".into(),
            Some(MangaSource::ComicAction),
            "".into(),
            db,
        )
        .await;
        result.unwrap();
    }

    #[tokio::test]
    async fn add_manga_success_ichijin_plus() {
        let db = get_test_db("add_manga_ichijin_plus").await.unwrap();
        let result = add_manga_service(
            "2550912965919360561".into(),
            Some(MangaSource::IchijinPlus),
            "".into(),
            db,
        )
        .await;
        result.unwrap();
    }

    #[tokio::test]
    async fn add_manga_success_mecha_comic() {
        let db = get_test_db("add_manga_mecha_comic").await.unwrap();
        let result = add_manga_service(
            "192830".into(),
            Some(MangaSource::MechaComic),
            "".into(),
            db,
        )
        .await;
        result.unwrap();
    }

    #[tokio::test]
    async fn add_manga_success_pocket_megazine() {
        let db = get_test_db("add_manga_pocket_megazine").await.unwrap();
        let result = add_manga_service(
            "2790".into(),
            Some(MangaSource::MagazinePocket),
            "".into(),
            db,
        )
        .await;
        result.unwrap();
    }

    #[tokio::test]
    async fn add_manga_success_urasunday() {
        let db = get_test_db("add_manga_urasunday").await.unwrap();
        let result = add_manga_service(
            "1707".into(),
            Some(MangaSource::Urasunday),
            get_selenium_driver_url().await,
            db,
        )
        .await;
        result.unwrap();
    }

    #[tokio::test]
    async fn add_manga_error_not_found() {
        let db = get_test_db("add_manga_404").await.unwrap();
        if (add_manga_service("".into(), Some(MangaSource::YoungAnimal), "".into(), db).await)
            .is_ok()
        {
            panic!("server fn should error")
        };
    }

    #[tokio::test]
    async fn delete_manga_error_not_found() {
        let db = get_test_db("delete_manga_not_found").await.unwrap();
        match delete_manga_service(vec![(MangaSource::TonariYoungJump, "1234".to_string())], db)
            .await
        {
            Ok(_) => panic!("server fn should error"),
            Err(err) => {
                assert_eq!(err.to_string(), "no manga deleted")
            }
        }
    }

    #[tokio::test]
    async fn delete_manga_success() {
        let id = "10834108156641784251";
        let db = get_test_db("delete_manga").await.unwrap();

        let _ = add_manga_service(
            id.to_string(),
            Some(MangaSource::ShounenJumpPlus),
            "".into(),
            db.clone(),
        )
        .await
        .unwrap();

        delete_manga_service(vec![(MangaSource::ShounenJumpPlus, id.to_string())], db)
            .await
            .unwrap();
    }
}
