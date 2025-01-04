use chrono::{Datelike, Local, NaiveDate, NaiveTime, TimeZone};
use chrono_tz::Japan;
use reqwest::Client;
use scraper::{selectable::Selectable, Html, Selector};

use crate::core::{fetch::FetchError, types::Manga};

pub fn parse_gamma_plus_from_html(html: String) -> Result<Manga, FetchError> {
    let header_selector = Selector::parse(r#"ul[class="manga__title"]"#).unwrap();
    let chapter_selector = Selector::parse(r#"div[class="read__outer"] > a"#).unwrap();
    let chapter_title_selector = Selector::parse(r#"li[class="episode"]"#).unwrap();
    let chapter_date_selector = Selector::parse(r#"li[class="episode__text"]"#).unwrap();
    let chapter_thumbnail_selector = Selector::parse(r#"img"#).unwrap();

    let document = Html::parse_document(&html);

    let mut header_children = document
        .select(&header_selector)
        .next()
        .ok_or(FetchError::PageNotFound(Some(
            "header section not found".into(),
        )))?
        .child_elements();

    let title = header_children
        .next()
        .ok_or(FetchError::PageNotFound(Some("title not found".into())))?
        .inner_html();

    let author = header_children
        .next()
        .ok_or(FetchError::PageNotFound(Some("author not found".into())))?
        .inner_html();

    let latest_chapter_element = document
        .select(&chapter_selector)
        .find(|d| d.attr("href").is_some_and(|t| t != "#comics"))
        .ok_or(FetchError::ChapterNotFound(Some(
            "chapter element not found".into(),
        )))?;

    let chapter_url = latest_chapter_element
        .attr("href")
        .ok_or(FetchError::ChapterNotFound(Some(
            "href attritbue not found".into(),
        )))?
        .replace("../../../", "https://gammaplus.takeshobo.co.jp/");

    let chapter_title = latest_chapter_element
        .select(&chapter_title_selector)
        .next()
        .ok_or(FetchError::ChapterNotFound(Some("title not found".into())))?
        .inner_html()
        .trim()
        .to_owned();

    let chapter_thumbnail = latest_chapter_element
        .select(&chapter_thumbnail_selector)
        .next()
        .ok_or(FetchError::ChapterNotFound(Some(
            "thumbnail element not found".into(),
        )))?
        .attr("src")
        .ok_or(FetchError::ChapterNotFound(Some(
            "thumbnail src attribute not found".into(),
        )))?
        .replace("../../", "https://gammaplus.takeshobo.co.jp/");

    let chapter_release_date = match latest_chapter_element.select(&chapter_date_selector).next() {
        Some(el) => {
            let naive_date =
                NaiveDate::parse_from_str(&el.inner_html(), "%Y年%m月%d日").map_err(|e| {
                    FetchError::ChapterNotFound(Some(format!(
                        "Error parsing date {} : {}",
                        &el.inner_html(),
                        e
                    )))
                })?;

            Local
                .from_local_datetime(&naive_date.and_time(NaiveTime::default()))
                .unwrap()
        }
        None => Local::now(),
    };

    Ok(Manga {
        title,
        cover_url: chapter_thumbnail,
        author,
        latest_chapter_title: chapter_title,
        latest_chapter_url: chapter_url,
        latest_chapter_release_date: chapter_release_date.fixed_offset(),
        latest_chapter_publish_day: chapter_release_date.with_timezone(&Japan).weekday(),
    })
}

pub async fn fetch_gamma_plus(client: Client, manga_id: &str) -> Result<Manga, FetchError> {
    let url = format!("https://gammaplus.takeshobo.co.jp/manga/{}", manga_id);

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

    parse_gamma_plus_from_html(html)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_parse_gamma_plus_source() {
        let paths = fs::read_dir("src/test_data/gamma_plus").unwrap();

        for path in paths {
            let html = fs::read_to_string(path.unwrap().path()).unwrap();
            let data = parse_gamma_plus_from_html(html).unwrap();
            dbg!(data);
        }
    }
}
