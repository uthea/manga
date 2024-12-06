use crate::core::types::{Manga, MangaSource};
use leptos::server;
use leptos::server_fn::ServerFnError;

#[cfg(feature = "ssr")]
use {
    crate::core::fetch::{fetch_manga, FetchError},
    crate::db::inquiry::get_manga,
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

    let manga = fetch_manga(&manga_id, source.as_ref().unwrap())
        .await
        .map_err(|e| {
            println!("Fetch error: {:?}", e);
            let msg = match e {
                FetchError::ConvertError(_) => "Error converting type",
                FetchError::ReqwestError(e) => {
                    if e.status()
                        .is_some_and(|s| s == reqwest::StatusCode::NOT_FOUND)
                    {
                        "Error page not found"
                    } else {
                        "Error on request"
                    }
                }
                FetchError::DeserialzeXmlError(_) => "Error deserializing rss",
                FetchError::YanmagaParseError(_) => "Error parsing yanmaga html",
                FetchError::UrasundayParseError(_) => "Error parsing urasunday html",
                FetchError::ComicPixivError(_) => "Error fetching data from comic pixiv api",
                FetchError::ComicWalkerError(_) => "Error fetching data from comic walker api",
                FetchError::MangaUpError(_) => "Error parseing mangaup html",
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
