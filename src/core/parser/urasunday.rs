use crate::core::{fetch::FetchError, types::Manga};
use chrono::{Datelike, Local, NaiveDate, NaiveTime, TimeZone};
use chrono_tz::Japan;
use reqwest::Client;
use scraper::{Html, Selector};

pub fn parse_urasunday_from_html(html: String) -> Result<Manga, FetchError> {
    let document = Html::parse_document(&html);

    let title_selector = Selector::parse(r#"div[class="info"] > h1"#).unwrap();
    let author_selector = Selector::parse(r#"div[class="author"]"#).unwrap();
    let chapter_selector = Selector::parse(r#"div[class="chapter"] > ul > li > a"#).unwrap();

    let title = document
        .select(&title_selector)
        .next()
        .ok_or(FetchError::PageNotFound(Some(
            "series title not found".into(),
        )))?
        .inner_html();
    let author = document
        .select(&author_selector)
        .next()
        .ok_or(FetchError::PageNotFound(Some("author not found".into())))?
        .inner_html();

    if let Some(lastest_chapter_fragment) = document.select(&chapter_selector).next() {
        let chapter_url =
            lastest_chapter_fragment
                .attr("href")
                .ok_or(FetchError::ChapterNotFound(Some(
                    "url not found in href attribute".into(),
                )))?;
        let mut chapter_childrens = lastest_chapter_fragment.child_elements();
        let chapter_img = chapter_childrens
            .next()
            .ok_or(FetchError::ChapterNotFound(Some("cover not found".into())))?
            .attr("src")
            .ok_or(FetchError::ChapterNotFound(Some(
                "cover not found in src attribute".into(),
            )))?;

        let chapter_details = chapter_childrens
            .next()
            .ok_or(FetchError::ChapterNotFound(Some(
                "chapter details is empty".into(),
            )))?;

        let mut chapter_details_children = chapter_details.child_elements();
        let chapter_title = {
            let first_title = chapter_details_children
                .next()
                .ok_or(FetchError::ChapterNotFound(Some(
                    "first title segment not found ".into(),
                )))?
                .inner_html();
            let second_title = chapter_details_children
                .next()
                .ok_or(FetchError::ChapterNotFound(Some(
                    "second title segment not found ".into(),
                )))?
                .inner_html();

            format!("{} {}", first_title, second_title)
        };

        let chapter_release_date = {
            let raw = chapter_details_children
                .last()
                .ok_or(FetchError::ChapterNotFound(Some(
                    "release date not found".into(),
                )))?
                .inner_html();

            let naive_date = NaiveDate::parse_from_str(&raw, "%Y/%m/%d").map_err(|e| {
                FetchError::ChapterNotFound(Some(format!("Error parsing date {} : {}", &raw, e)))
            })?;

            Local
                .from_local_datetime(&naive_date.and_time(NaiveTime::default()))
                .unwrap()
        };

        return Ok(Manga {
            title: title.trim().into(),
            cover_url: chapter_img.into(),
            author: author.trim().into(),
            latest_chapter_title: chapter_title,
            latest_chapter_url: format!("https://urasunday.com{}", chapter_url),
            latest_chapter_release_date: chapter_release_date.fixed_offset(),
            latest_chapter_publish_day: chapter_release_date.with_timezone(&Japan).weekday(),
        });
    }

    Err(FetchError::ChapterNotFound(None))
}

pub async fn fetch_urasunday(client: Client, manga_id: &str) -> Result<Manga, FetchError> {
    let url = format!("https://urasunday.com/title/{}", manga_id);

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

    parse_urasunday_from_html(html)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_parse_urasunday_source() {
        let paths = fs::read_dir("src/test_data/urasunday").unwrap();

        for path in paths {
            dbg!(&path);
            let html = fs::read_to_string(path.unwrap().path()).unwrap();
            let _ = parse_urasunday_from_html(html).unwrap();
        }
    }
}
