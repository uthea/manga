use chrono::{Datelike, Local};
use chrono_tz::Japan;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use crate::core::{fetch::FetchError, types::Manga};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GanganOnline {
    pub props: Props,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
    pub page_props: PageProps,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageProps {
    pub data: Data,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub default: Default,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Default {
    pub chapters: Vec<Chapter>,
    pub title_name: String,
    pub image_url: String,
    pub author: String,
    pub description: String,
    pub impression_url: String,
    pub title_id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chapter {
    pub id: i64,
    pub status: Option<i64>,
    pub thumbnail_url: String,
    pub main_text: String,
    pub sub_text: Option<String>,
    pub app_launch_url: Option<String>,
    pub publishing_period: Option<String>,
}

pub fn parse_gangan_online_from_html(html: String) -> Result<Manga, FetchError> {
    let next_data_selector = Selector::parse(r#"script[id="__NEXT_DATA__"]"#).unwrap();
    let document = Html::parse_document(&html);

    let next_data = document
        .select(&next_data_selector)
        .next()
        .ok_or(FetchError::PageNotFound(Some(
            "__NEXT_DATA__ not found".into(),
        )))?
        .inner_html();

    let data = {
        let obj: GanganOnline = serde_json::from_str(&next_data).map_err(|e| {
            dbg!(&e);
            FetchError::JsonDeserializeError(e)
        })?;

        obj.props.page_props.data.default
    };

    let latest_chapter = data
        .chapters
        .first()
        .ok_or(FetchError::PageNotFound(Some("chapters are empty".into())))?;

    let latest_chapter_title = if latest_chapter.sub_text.is_some() {
        latest_chapter.sub_text.clone().unwrap()
    } else {
        latest_chapter.main_text.to_owned()
    };

    Ok(Manga {
        title: data.title_name.to_owned(),
        cover_url: format!(
            "https://www.ganganonline.com{}",
            latest_chapter.thumbnail_url
        ),
        author: data.author.to_owned(),
        latest_chapter_title,
        latest_chapter_url: format!(
            "https://www.ganganonline.com/title/{}/chapter/{}",
            data.title_id, latest_chapter.id
        ),
        latest_chapter_release_date: Local::now().fixed_offset(),
        latest_chapter_publish_day: Local::now().with_timezone(&Japan).weekday(),
    })
}

pub async fn fetch_gangan_online(client: Client, manga_id: &str) -> Result<Manga, FetchError> {
    let url = format!("https://www.ganganonline.com/title/{manga_id}");

    let html = client
        .get(url)
        .send()
        .await
        .map_err(FetchError::ReqwestError)?
        .error_for_status()
        .map_err(FetchError::ReqwestError)?
        .text()
        .await
        .map_err(FetchError::ReqwestError)?;

    parse_gangan_online_from_html(html)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_parse_gangan_online_source() {
        let paths = fs::read_dir("src/test_data/gangan_online").unwrap();

        for path in paths {
            let html = fs::read_to_string(path.unwrap().path()).unwrap();
            let data = parse_gangan_online_from_html(html).unwrap();
            dbg!(data);
        }
    }
}
