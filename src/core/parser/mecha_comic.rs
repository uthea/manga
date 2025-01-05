use chrono::{Datelike, Local};
use reqwest::Client;
use scraper::{selectable::Selectable, Html, Selector};

use crate::core::{fetch::FetchError, types::Manga};

fn parse_mecha_comic_from_html(html: String) -> Result<Manga, FetchError> {
    let title_selector = Selector::parse(r#"div[class="p-bookInfo_title"] > h1"#).unwrap();
    let cover_selector =
        Selector::parse(r#"div[class="p-bookInfo_jacket"] > img[class="jacket_image_l"]"#).unwrap();
    let author_selector =
        Selector::parse(r#"span[class="p-sepList_item p-sepList_item-thrash"] > a"#).unwrap();
    let chapter_section_selector =
        Selector::parse(r#"div[class="p-chapterInfo p-chapterInfo-comic u-clearfix"]"#).unwrap();
    let chapter_num_selector = Selector::parse(r#"dt[class="p-chapterList_no"]"#).unwrap();
    let chapter_title_selector = Selector::parse(r#"dd[class="p-chapterList_name"]"#).unwrap();
    let chapter_url_selector =
        Selector::parse(r#"a[class="p-btn-chapter c-btn c-btn-boder-buy prevent"]"#).unwrap();

    let document = Html::parse_document(&html);

    let title = document
        .select(&title_selector)
        .next()
        .ok_or(FetchError::PageNotFound(Some("title not found".into())))?
        .inner_html();

    let cover_url = document
        .select(&cover_selector)
        .next()
        .ok_or(FetchError::PageNotFound(Some(
            "cover element not found".into(),
        )))?
        .attr("src")
        .ok_or(FetchError::PageNotFound(Some("cover src is empty".into())))?;

    let author = document
        .select(&author_selector)
        .next()
        .ok_or(FetchError::PageNotFound(Some("author not found".into())))?
        .inner_html();

    if let Some(latest_chapter_element) = document.select(&chapter_section_selector).last() {
        let chapter_num = latest_chapter_element
            .select(&chapter_num_selector)
            .next()
            .ok_or(FetchError::ChapterNotFound(Some(
                "chapter num element not found".into(),
            )))?
            .first_child()
            .ok_or(FetchError::ChapterNotFound(Some(
                "chapter num child is empty".into(),
            )))?
            .value()
            .as_text()
            .ok_or(FetchError::ChapterNotFound(Some(
                "chapter num child is not text".into(),
            )))?
            .trim();

        let chapter_title = latest_chapter_element
            .select(&chapter_title_selector)
            .next()
            .ok_or(FetchError::ChapterNotFound(Some(
                "chapter title not found".into(),
            )))?
            .inner_html();

        let chapter_url = latest_chapter_element
            .select(&chapter_url_selector)
            .next()
            .ok_or(FetchError::ChapterNotFound(Some(
                "chapter url not found".into(),
            )))?
            .attr("href")
            .ok_or(FetchError::ChapterNotFound(Some(
                "chapter url href attribute is empty".into(),
            )))?;

        // no information about chapter release date
        return Ok(Manga {
            title,
            cover_url: cover_url.to_owned(),
            author,
            latest_chapter_title: format!("{} {}", chapter_num, chapter_title.trim()),
            latest_chapter_url: format!("https://mechacomic.jp{}", chapter_url),
            latest_chapter_release_date: Local::now().fixed_offset(),
            latest_chapter_publish_day: Local::now().weekday(),
        });
    }

    Err(FetchError::ChapterNotFound(Some(
        "chapter section element not found".into(),
    )))
}

fn find_latest_chapter_number(html: String) -> Result<i32, FetchError> {
    let chapter_number_selector = Selector::parse(r#"div[class="u-inlineBlock"] > span"#).unwrap();
    let document = Html::parse_document(&html);

    document
        .select(&chapter_number_selector)
        .next()
        .ok_or(FetchError::PageNotFound(Some(
            "no match for chapter number selector".into(),
        )))?
        .inner_html()
        .replace("／", "")
        .replace("話へ", "")
        .parse()
        .map_err(|e| FetchError::PageNotFound(Some(format!("Page number parse error: {}", e))))
}

pub async fn fetch_mecha_comic(client: Client, manga_id: &str) -> Result<Manga, FetchError> {
    let url = format!("https://mechacomic.jp/books/{}", manga_id);

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

    let latest_chap_num = find_latest_chapter_number(html)?;

    let html = client
        .get(url)
        .query(&[("chapter_number", latest_chap_num)])
        .send()
        .await
        .map_err(FetchError::ReqwestError)?
        .error_for_status()
        .map_err(FetchError::ReqwestError)?
        .text()
        .await
        .map_err(FetchError::ReqwestError)?;

    parse_mecha_comic_from_html(html)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_parse_mecha_comic_source() {
        let paths = fs::read_dir("src/test_data/mecha_comic").unwrap();

        for path in paths {
            dbg!(&path);
            let html = fs::read_to_string(path.unwrap().path()).unwrap();
            let _ = find_latest_chapter_number(html.clone()).unwrap();
            let _ = parse_mecha_comic_from_html(html).unwrap();
        }
    }
}
