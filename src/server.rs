use crate::core::types::Paginated;
use crate::core::types::{Manga, MangaQuery, MangaSource};
use leptos::server;
use leptos::server_fn::ServerFnError;

#[cfg(feature = "ssr")]
use {
    crate::core::fetch::FetchError,
    crate::db::inquiry::{get_manga, get_manga_paginated},
    crate::db::insert::insert_manga,
    crate::state::AppState,
    leptos::context::use_context,
};

#[server]
pub async fn add_manga(
    manga_id: String,
    source: Option<MangaSource>,
) -> Result<Manga, ServerFnError> {
    let state = use_context::<AppState>().expect("AppState not found from context");

    if source.is_none() {
        return Err(ServerFnError::new("Source cannot be empty"));
    }

    // check if manga exist or not
    let check_exist = get_manga(source.as_ref().unwrap(), &manga_id, &state.pool).await;

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
    insert_manga(source.unwrap(), manga_id, manga.clone(), &state.pool)
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
) -> Result<Paginated<Vec<(MangaSource, Manga)>>, ServerFnError> {
    let state = use_context::<AppState>().expect("AppState not found from context");

    let paginated_result = get_manga_paginated(page_number, page_size, query_option, &state.pool)
        .await
        .map_err(|_| ServerFnError::new("Error at querying manga"))?;

    let result = Paginated {
        data: paginated_result
            .data
            .into_iter()
            .map(|d| (d.source.clone(), d.into_manga()))
            .collect(),
        total_page: paginated_result.total_page,
    };

    Ok(result)
}
