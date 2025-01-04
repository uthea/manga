use chrono::DateTime;
use chrono::Datelike;
use chrono_tz::Japan;
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;

use crate::core::fetch::FetchError;
use crate::core::types::Manga;

// metadata struct
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub data: Data,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    #[serde(rename = "official_work")]
    pub official_work: OfficialWork,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficialWork {
    pub id: i64,
    pub name: String,
    pub author: String,
    pub description: String,
}

// detail struct
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Detail {
    pub data: EpisodeData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpisodeData {
    pub episodes: Vec<Episode>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Episode {
    pub state: String,
    pub episode: Option<EpisodeDetail>,
    pub message: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpisodeDetail {
    pub id: i64,
    #[serde(rename = "numbering_title")]
    pub numbering_title: String,
    #[serde(rename = "sub_title")]
    pub sub_title: String,
    #[serde(rename = "read_start_at")]
    pub read_start_at: i64,
    #[serde(rename = "thumbnail_image_url")]
    pub thumbnail_image_url: String,
    #[serde(rename = "viewer_path")]
    pub viewer_path: String,
    #[serde(rename = "is_tateyomi")]
    pub is_tateyomi: bool,
    #[serde(rename = "sales_type")]
    pub sales_type: String,
    #[serde(rename = "is_purchased")]
    pub is_purchased: bool,
    pub state: String,
}

pub async fn fetch_pixiv_data(client: Client, id: &str) -> Result<Manga, FetchError> {
    let metadata = client
        .get(format!("https://comic.pixiv.net/api/app/works/v5/{}", id))
        .header("x-requested-with", "pixivcomic")
        .send()
        .await
        .map_err(FetchError::ReqwestError)?
        .json::<Metadata>()
        .await
        .map_err(FetchError::ReqwestError)?;

    let details = client
        .get(format!(
            "https://comic.pixiv.net/api/app/works/{}/episodes/v2?order=desc",
            id
        ))
        .header("x-requested-with", "pixivcomic")
        .send()
        .await
        .map_err(FetchError::ReqwestError)?
        .json::<Detail>()
        .await
        .map_err(FetchError::ReqwestError)?;

    if details.data.episodes.is_empty() {
        return Err(FetchError::ChapterNotFound(Some(
            "episodes is empty".into(),
        )));
    }

    let latest_episode = details
        .data
        .episodes
        .into_iter()
        .find(|d| d.state.ne("not_publishing"))
        .ok_or(FetchError::ChapterNotFound(Some(
            "latest episode not found".into(),
        )))?;

    let episode_detail = latest_episode.episode.unwrap();

    let release_date = DateTime::from_timestamp_millis(episode_detail.read_start_at)
        .unwrap()
        .with_timezone(&Japan);

    Ok(Manga {
        title: metadata.data.official_work.name,
        cover_url: episode_detail.thumbnail_image_url.clone(),
        author: metadata.data.official_work.author,
        latest_chapter_title: episode_detail.numbering_title.clone(),
        latest_chapter_url: format!("https://comic.pixiv.net{}", episode_detail.viewer_path),
        latest_chapter_release_date: release_date.fixed_offset(),
        latest_chapter_publish_day: release_date.weekday(),
    })
}
