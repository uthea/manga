use chrono::{Datelike, Local};
use reqwest::Client;
use scraper::{Html, Selector};

use crate::core::{fetch::FetchError, types::Manga};

pub fn parse_ganma_from_html(html: String) -> Result<Manga, FetchError> {
    let title_selector =
        Selector::parse(r#"h2[class="text-lg font-semibold leading-tight"]"#).unwrap();
    let author_selector = Selector::parse(r#"div[class="font-semibold"]"#).unwrap();
    let cover_selector = Selector::parse(r#"img[class="pointer-events-none"]"#).unwrap();
    let total_chapter_selector =
        Selector::parse(r#"span[class="text-g-black font-semibold"]"#).unwrap();
    let chapter_url_selector =
        Selector::parse(r#"a[class="flex items-center justify-center gap-1 p-4"]"#).unwrap();

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

    // latest chapter only include total chapter number
    let cover_url = document
        .select(&cover_selector)
        .next()
        .ok_or(FetchError::ChapterNotFound(Some("cover not found".into())))?
        .attr("src")
        .ok_or(FetchError::ChapterNotFound(Some(
            "cover src attribute not found".into(),
        )))?; // use series thumbnail
    let total_chapter_count = document
        .select(&total_chapter_selector)
        .next()
        .ok_or(FetchError::ChapterNotFound(Some(
            "chapter count not found".into(),
        )))?
        .inner_html();
    let chapter_url = document
        .select(&chapter_url_selector)
        .next()
        .ok_or(FetchError::ChapterNotFound(Some("url not found".into())))?
        .attr("href")
        .ok_or(FetchError::ChapterNotFound(Some(
            "url href attribute not found".into(),
        )))?; // the url redirect to app store / play store

    Ok(Manga {
        title,
        cover_url: cover_url.to_owned(),
        author,
        latest_chapter_title: total_chapter_count.replace("<!-- -->", ""),
        latest_chapter_url: chapter_url.to_owned(),
        latest_chapter_release_date: Local::now().fixed_offset(),
        latest_chapter_publish_day: Local::now().weekday(),
    })
}

pub async fn fetch_ganma(client: Client, manga_id: &str) -> Result<Manga, FetchError> {
    let url = format!("https://ganma.jp/web/magazine/{}", manga_id);

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

    parse_ganma_from_html(html)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_parse_ganma_source() {
        let paths = fs::read_dir("src/test_data/ganma").unwrap();

        for path in paths {
            let html = fs::read_to_string(path.unwrap().path()).unwrap();
            let data = parse_ganma_from_html(html).unwrap();
            dbg!(data);
        }
    }
}
