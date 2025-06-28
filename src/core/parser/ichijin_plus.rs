use chrono::{DateTime, Datelike, Local};
use chrono_tz::Japan;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::core::{fetch::FetchError, types::Manga};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IchijinPlusData {
    pub authors: Vec<Author>,
    pub title: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    #[serde(rename = "creator_id")]
    pub creator_id: String,
    pub name: String,
    pub role: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpisodeDetail {
    pub resources: Vec<Episode>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Episode {
    #[serde(rename = "comic_id")]
    pub comic_id: String,
    pub id: String,
    #[serde(rename = "published_at")]
    pub published_at: DateTime<Local>,
    #[serde(rename = "thumbnail_image_url")]
    pub thumbnail_image_url: String,
    pub title: String,
}

const HEADER_KEY: &str = "x-api-environment-key";
const HEADER_VALUE: &str = "GGXGejnSsZw-IxHKQp8OQKHH-NDItSbEq5PU0g2w1W4=";

pub async fn fetch_ichijin_plus_data(client: Client, id: &str) -> Result<Manga, FetchError> {
    let data = client
        .get(format!("https://api.ichijin-plus.com/comics/{id}"))
        .header(HEADER_KEY, HEADER_VALUE)
        .send()
        .await
        .map_err(FetchError::ReqwestError)?
        .json::<IchijinPlusData>()
        .await
        .map_err(FetchError::ReqwestError)?;

    let episode_detail = client
        .get(format!("https://api.ichijin-plus.com/episodes?comic_id={id}&episode_status=free_viewing%2Con_sale&limit=5&order=desc&sort=episode_order"))
        .header(
            HEADER_KEY,
            HEADER_VALUE,
        )
        .send()
        .await
        .map_err(FetchError::ReqwestError)?
        .json::<EpisodeDetail>()
        .await
        .map_err(FetchError::ReqwestError)?;

    let latest_episode = episode_detail
        .resources
        .first()
        .ok_or(FetchError::ChapterNotFound(Some("No episode found".into())))?;

    let author = data
        .authors
        .into_iter()
        .map(|d| d.name)
        .collect::<Vec<_>>()
        .join(",");

    Ok(Manga {
        title: data.title,
        cover_url: latest_episode.thumbnail_image_url.clone(),
        author,
        latest_chapter_title: latest_episode.title.clone(),
        latest_chapter_url: format!("https://ichijin-plus.com/episodes/{}", &latest_episode.id),
        latest_chapter_release_date: latest_episode.published_at.fixed_offset(),
        latest_chapter_publish_day: latest_episode.published_at.with_timezone(&Japan).weekday(),
    })
}
