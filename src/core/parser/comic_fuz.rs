use chrono::{Datelike, Local, NaiveDate, NaiveTime, TimeZone};
use chrono_tz::Japan;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use crate::core::{fetch::FetchError, types::Manga};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComicFuz {
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
    pub chapters: Vec<Chapter>,
    pub authorships: Vec<Authorship>,
    pub manga: MangaInfo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chapter {
    pub chapters: Vec<ChapterDetail>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterDetail {
    pub chapter_id: i64,
    pub chapter_main_name: String,
    pub thumbnail_url: String,
    pub updated_date: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Authorship {
    pub author: Author,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub author_id: i64,
    pub author_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MangaInfo {
    pub manga_id: i64,
    pub manga_name: String,
}

pub fn parse_comic_fuz_from_html(html: String) -> Result<Manga, FetchError> {
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
        let obj: ComicFuz = serde_json::from_str(&next_data).map_err(|e| {
            dbg!(&e);
            FetchError::PageNotFound(Some(e.to_string()))
        })?;
        obj.props.page_props
    };

    let author = data
        .authorships
        .into_iter()
        .map(|d| d.author.author_name.to_string())
        .collect::<Vec<_>>()
        .join(",");

    let latest_chapter = data
        .chapters
        .first()
        .ok_or(FetchError::ChapterNotFound(Some(
            "chapters is empty".into(),
        )))?
        .chapters
        .first()
        .ok_or(FetchError::ChapterNotFound(Some(
            "nested chapters is empty".into(),
        )))?;

    let release_date = match &latest_chapter.updated_date {
        Some(raw) => {
            let naive_date = NaiveDate::parse_from_str(raw, "%Y/%m/%d").map_err(|e| {
                FetchError::ChapterNotFound(Some(format!("error on date parse {} : {}", &raw, e)))
            })?;

            Local
                .from_local_datetime(&naive_date.and_time(NaiveTime::default()))
                .unwrap()
        }
        None => Local::now(),
    };

    Ok(Manga {
        title: data.manga.manga_name.to_owned(),
        cover_url: format!("https://img.comic-fuz.com{}", latest_chapter.thumbnail_url),
        author,
        latest_chapter_title: latest_chapter.chapter_main_name.to_owned(),
        latest_chapter_url: format!(
            "https://comic-fuz.com/manga/viewer/{}",
            latest_chapter.chapter_id
        ),
        latest_chapter_release_date: release_date.fixed_offset(),
        latest_chapter_publish_day: release_date.with_timezone(&Japan).weekday(),
    })
}

pub async fn fetch_comic_fuz(client: Client, manga_id: &str) -> Result<Manga, FetchError> {
    let url = format!("https://comic-fuz.com/manga/{manga_id}");

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

    parse_comic_fuz_from_html(html)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_parse_comic_fuz_source() {
        let paths = fs::read_dir("src/test_data/comic_fuz").unwrap();

        for path in paths {
            let html = fs::read_to_string(path.unwrap().path()).unwrap();
            let data = parse_comic_fuz_from_html(html).unwrap();
            dbg!(data);
        }
    }
}
