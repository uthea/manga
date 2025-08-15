use crate::core::{fetch::FetchError, types::Manga};
use chrono::{Datelike, Local};
use regex::{Regex, RegexBuilder};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::{sync::LazyLock, time::Duration};

static CHAPTER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(r#"\{\\"titleName\\.*currentChapter.*\],\[\\"\$\\",\\"\$L6f"#)
        .multi_line(true)
        .build()
        .unwrap()
});

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MangaUpData {
    pub title_name: String,
    pub title_id: i64,
    pub chapters: Vec<Chapter>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chapter {
    pub id: i64,
    pub name: String,
    pub sub_name: String,
    pub can_comment: bool,
    pub comment_count: i64,
    pub url_thumbnail: String,
    pub status: i64,
    pub days_to_change_status: String,
}

pub fn parse_manga_up_from_html(html: String) -> Result<Manga, FetchError> {
    let title_selector = Selector::parse(r#"h2[class*="pc:text-title-lg-pc"]"#).unwrap();
    let author_selector = Selector::parse(
        r#"div[class="text-on_background_medium sp:text-body-md-sp pc:text-body-md-pc"]"#,
    )
    .unwrap();

    let document = Html::parse_document(&html);
    let title = document
        .select(&title_selector)
        .next()
        .ok_or(FetchError::PageNotFound(Some("title not found".into())))?
        .inner_html();
    let author = document
        .select(&author_selector)
        .next()
        .ok_or(FetchError::PageNotFound(Some("author not found".into())))?
        .inner_html();

    // parse chapter related data
    let captures = CHAPTER_REGEX
        .captures(&html)
        .ok_or(FetchError::ChapterNotFound(Some(
            "no match from regex search".into(),
        )))?;

    let chapter_data_json = captures
        .get(0)
        .ok_or(FetchError::ChapterNotFound(Some(
            "no result from regex capture".into(),
        )))?
        .as_str()
        .replace(r#"],[\"$\",\"$L6f"#, "")
        .replace("\\", "");

    let chapter_data: MangaUpData =
        serde_json::from_str(&chapter_data_json).map_err(FetchError::JsonDeserializeError)?;

    let latest_chapter = chapter_data
        .chapters
        .last()
        .ok_or(FetchError::ChapterNotFound(Some(
            "chapters is empty".into(),
        )))?;

    Ok(Manga {
        title,
        cover_url: latest_chapter.url_thumbnail.to_owned(),
        author,
        latest_chapter_title: format!("{} {}", latest_chapter.sub_name, latest_chapter.name)
            .trim()
            .to_owned(),
        latest_chapter_url: format!(
            "https://www.manga-up.com/titles/{}/chapters/{}",
            chapter_data.title_id, latest_chapter.id
        ),
        latest_chapter_release_date: Local::now().fixed_offset(),
        latest_chapter_publish_day: Local::now().weekday(),
    })
}

pub async fn fetch_mangaup(client: Client, manga_id: &str) -> Result<Manga, FetchError> {
    let url = format!("https://www.manga-up.com/titles/{manga_id}");

    let mut counter = 0;

    // retry until counter or html is not empty
    loop {
        let html = client
            .get(&url)
            .send()
            .await
            .map_err(FetchError::ReqwestError)?
            .error_for_status()
            .map_err(FetchError::ReqwestError)?
            .text()
            .await
            .map_err(FetchError::ReqwestError)?;

        if !html.is_empty() || counter > 10 {
            if html.is_empty() {
                println!("MANGA UP: retry exceed max retry and still return empty html")
            }
            let result = parse_manga_up_from_html(html);
            return result;
        }

        counter += 1;
        println!("MANGA UP return empty html, retry attempt: {counter}");
        tokio::time::sleep(Duration::from_millis(500))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_parse_manga_up_source() {
        let paths = fs::read_dir("src/test_data/manga_up").unwrap();

        for path in paths {
            let html = fs::read_to_string(path.unwrap().path()).unwrap();
            let data = parse_manga_up_from_html(html).unwrap();
            dbg!(data);
        }
    }
}
