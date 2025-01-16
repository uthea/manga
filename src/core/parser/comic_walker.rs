use chrono::DateTime;
use chrono::Datelike;
use chrono::Local;
use chrono_tz::Japan;
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;

use crate::core::fetch::FetchError;
use crate::core::types::Manga;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComicWalkerData {
    pub work: Work,
    pub latest_episodes: LatestEpisodes,
    pub latest_episode_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Work {
    pub code: String,
    pub id: String,
    pub thumbnail: String,
    pub original_thumbnail: String,
    pub book_cover: Option<String>,
    pub title: String,
    pub language: String,
    pub serialization_status: String,
    pub summary: String,
    pub authors: Vec<Author>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub id: String,
    pub name: String,
    pub role: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestEpisodes {
    pub total: i64,
    pub result: Vec<EpisodeResult>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", rename = "result")]
pub struct EpisodeResult {
    pub id: String,
    pub code: String,
    pub title: String,
    pub sub_title: String,
    pub thumbnail: Option<String>,
    pub original_thumbnail: Option<String>,
    pub update_date: DateTime<Local>,
    pub delivery_period: String,
    pub is_new: bool,
    pub has_read: bool,
    pub service_id: String,
    pub internal: Internal,
    #[serde(rename = "type")]
    pub type_field: String,
    pub is_active: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Internal {
    pub episode_no: i64,
    pub page_count: i64,
    pub episodetype: String,
}

pub async fn fetch_comic_walker_data(client: Client, id: &str) -> Result<Manga, FetchError> {
    let data = client
        .get(format!(
            "https://comic-walker.com/api/contents/details/work?workCode={}",
            id
        ))
/*         .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36") */
        .send()
        .await
        .map_err(FetchError::ReqwestError)?
        .json::<ComicWalkerData>()
        .await
        .map_err(FetchError::ReqwestError)?;

    let latest_chapter = data
        .latest_episodes
        .result
        .first()
        .ok_or(FetchError::ChapterNotFound(Some(
            "episodes is empty".into(),
        )))?;

    let author = data
        .work
        .authors
        .iter()
        .map(|d| d.name.to_owned())
        .collect::<Vec<_>>()
        .join(",");

    let release_date = latest_chapter.update_date.with_timezone(&Japan);

    Ok(Manga {
        title: data.work.title,
        cover_url: latest_chapter
            .original_thumbnail
            .to_owned()
            .unwrap_or(data.work.original_thumbnail.clone()),
        author,
        latest_chapter_title: latest_chapter.title.to_owned(),
        latest_chapter_url: format!(
            "https://comic-walker.com/detail/{}/episodes/{}",
            id, &latest_chapter.code
        ),
        latest_chapter_release_date: release_date.fixed_offset(),
        latest_chapter_publish_day: release_date.weekday(),
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_deserialize_comic_walker_source() {
        let paths = fs::read_dir("src/test_data/comic_walker").unwrap();

        for path in paths {
            let json = fs::read_to_string(path.unwrap().path()).unwrap();
            let data: ComicWalkerData = serde_json::from_str(&json).unwrap();
            dbg!(data);
        }
    }
}
