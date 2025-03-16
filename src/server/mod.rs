use crate::core::types::Paginated;
use crate::core::types::{Manga, MangaQuery, MangaSource};
use leptos::server;
use leptos::server_fn::ServerFnError;

#[cfg(feature = "ssr")]
pub mod service;

#[cfg(feature = "ssr")]
use {
    service::{add_manga_service, delete_manga_service, retrieve_manga_service},
    sqlx::Pool,
    sqlx::Postgres,
};

#[cfg(feature = "ssr")]
async fn get_db() -> Result<Pool<Postgres>, ServerFnError> {
    use crate::state::AppState;
    use leptos::prelude::use_context;

    let db = use_context::<AppState>()
        .ok_or(ServerFnError::new("AppState not found from context"))?
        .pool;

    Ok(db)
}

#[server]
pub async fn add_manga(
    manga_id: String,
    source: Option<MangaSource>,
) -> Result<Manga, ServerFnError> {
    let db = get_db().await?;

    add_manga_service(manga_id, source, db)
        .await
        .map_err(ServerFnError::new)
}

#[server]
pub async fn retrieve_manga(
    page_number: i64,
    page_size: i64,
    #[server(default)] query_option: MangaQuery,
) -> Result<Paginated<Vec<(MangaSource, String, Manga)>>, ServerFnError> {
    let db = get_db().await?;

    retrieve_manga_service(page_number, page_size, query_option, db)
        .await
        .map_err(ServerFnError::new)
}

#[server]
pub async fn delete_manga(
    #[server(default)] manga_list: Vec<(MangaSource, String)>,
) -> Result<u64, ServerFnError> {
    let db = get_db().await?;

    delete_manga_service(manga_list, db)
        .await
        .map_err(ServerFnError::new)
}
