use chrono::{Datelike, Local, NaiveDate, NaiveTime, TimeZone};
use chrono_tz::Japan;
use reqwest::Client;
use scraper::{selectable::Selectable, Html, Selector};

use crate::core::{fetch::FetchError, types::Manga};

pub fn parse_yanmaga_from_html(html: String) -> Result<Manga, FetchError> {
    let document = Html::parse_document(&html);
    let title_selector = Selector::parse(r#"h1[class="detailv2-outline-title"]"#).unwrap();
    let author_selector =
        Selector::parse(r#"li[class="detailv2-outline-author-item"] > a > h2"#).unwrap();

    // chapter related selector
    let chapter_selector = Selector::parse(r#"li[class="mod-episode-item"]"#).unwrap();
    let cover_url_selector =
        Selector::parse(r#"div[class="mod-episode-thumbnail-image"] > img"#).unwrap();
    let chapter_title_selector = Selector::parse(r#"p[class="mod-episode-title"]"#).unwrap();
    let chapter_url_selector = Selector::parse(r#"a[class="mod-episode-link    "#).unwrap();
    let chapter_release_date_selector =
        Selector::parse(r#"time[class="mod-episode-date"]"#).unwrap();

    // when the chapter is unreleased  (inside chapter_selector)
    // first child : title
    // second child : release date
    let chapter_not_released_selector =
        Selector::parse(r#"p[class*="mod-episode-date-before-publication"]"#).unwrap();

    // query document

    let title = document
        .select(&title_selector)
        .next()
        .ok_or(FetchError::PageNotFound(Some("Title not found".into())))?
        .inner_html();
    let author = document
        .select(&author_selector)
        .next()
        .ok_or(FetchError::PageNotFound(Some("Author not found".into())))?
        .inner_html();

    // get latest chapter
    let latest_chapter =
        document
            .select(&chapter_selector)
            .next()
            .ok_or(FetchError::ChapterNotFound(Some(
                "zero result from chapter selector".into(),
            )))?;

    if let Some(not_released) = latest_chapter.select(&chapter_not_released_selector).next() {
        let mut childrens = not_released.child_elements();
        let chapter_title = childrens
            .next()
            .ok_or(FetchError::ChapterNotFound(Some("title not found".into())))?
            .inner_html();
        let chapter_release_date = {
            let raw_date = childrens
                .next()
                .ok_or(FetchError::ChapterNotFound(Some(
                    "release date not found".into(),
                )))?
                .inner_html();
            let date_only = raw_date
                .split('(')
                .next()
                .ok_or(FetchError::ChapterNotFound(Some(format!(
                    "error extracting date from : {}",
                    &raw_date
                ))))?;
            let naive_date = NaiveDate::parse_from_str(date_only, "%Y/%m/%d").map_err(|e| {
                FetchError::ChapterNotFound(Some(format!(
                    "{}, error parsing date : {}",
                    e, date_only
                )))
            })?;

            Local
                .from_local_datetime(&naive_date.and_time(NaiveTime::default()))
                .unwrap()
        };

        let cover_url = latest_chapter
            .select(&cover_url_selector)
            .next()
            .ok_or(FetchError::ChapterNotFound(Some("cover not found".into())))?
            .attr("src")
            .ok_or(FetchError::ChapterNotFound(Some(
                "src attribute is empty".into(),
            )))?;

        Ok(Manga {
            title,
            cover_url: cover_url.to_string(),
            author,
            latest_chapter_title: chapter_title,
            latest_chapter_url: "".into(),
            latest_chapter_release_date: chapter_release_date.into(),
            latest_chapter_publish_day: chapter_release_date.with_timezone(&Japan).weekday(),
        })
    } else {
        let chapter_title = latest_chapter
            .select(&chapter_title_selector)
            .next()
            .ok_or(FetchError::ChapterNotFound(Some("title not found".into())))?
            .inner_html();

        let chapter_release_date = NaiveDate::parse_from_str(
            latest_chapter
                .select(&chapter_release_date_selector)
                .next()
                .ok_or(FetchError::ChapterNotFound(Some(
                    "release date not found".into(),
                )))?
                .inner_html()
                .as_str(),
            "%Y/%m/%d",
        )
        .map_err(|e| FetchError::ChapterNotFound(Some(format!("error parsing date : {}", e))))?;

        let chapter_release_date = Local
            .from_local_datetime(&chapter_release_date.and_time(NaiveTime::default()))
            .unwrap();

        let cover_url = latest_chapter
            .select(&cover_url_selector)
            .next()
            .ok_or(FetchError::ChapterNotFound(Some("cover not found".into())))?
            .attr("src")
            .ok_or(FetchError::ChapterNotFound(Some(
                "src attribute is empty".into(),
            )))?;

        let chapter_url = latest_chapter
            .select(&chapter_url_selector)
            .next()
            .ok_or(FetchError::ChapterNotFound(Some("url not found".into())))?
            .attr("href")
            .ok_or(FetchError::ChapterNotFound(Some(
                "href attribute is empty".into(),
            )))?;

        Ok(Manga {
            title,
            cover_url: cover_url.to_string(),
            author,
            latest_chapter_title: chapter_title,
            latest_chapter_url: format!("https://yanmaga.jp{}", chapter_url),
            latest_chapter_release_date: chapter_release_date.into(),
            latest_chapter_publish_day: chapter_release_date.with_timezone(&Japan).weekday(),
        })
    }
}

pub async fn fetch_yanmaga(client: Client, manga_id: &str) -> Result<Manga, FetchError> {
    let url = format!("https://yanmaga.jp/comics/{}", manga_id);

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

    parse_yanmaga_from_html(html)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_parse_yanmaga_source() {
        let paths = fs::read_dir("src/test_data/yanmaga").unwrap();

        for path in paths {
            dbg!(&path);
            let html = fs::read_to_string(path.unwrap().path()).unwrap();
            let _ = parse_yanmaga_from_html(html).unwrap();
        }
    }
}
