use crate::core::types::Manga;
use chrono::{Datelike, Local, NaiveDate, NaiveTime, TimeZone};
use chrono_tz::Japan;
use scraper::{Html, Selector};

#[derive(Debug)]
pub enum UrasundayParseError {
    NotFound,
    ChapterNotFound,
}

pub fn parse_urasunday_from_html(html: String) -> Result<Manga, UrasundayParseError> {
    let document = Html::parse_document(&html);

    let title_selector = Selector::parse(r#"div[class="info"] > h1"#).unwrap();
    let author_selector = Selector::parse(r#"div[class="author"]"#).unwrap();
    let chapter_selector = Selector::parse(r#"div[class="chapter"] > ul > li > a"#).unwrap();

    let title = document
        .select(&title_selector)
        .next()
        .ok_or(UrasundayParseError::NotFound)?
        .inner_html();
    let author = document
        .select(&author_selector)
        .next()
        .ok_or(UrasundayParseError::NotFound)?
        .inner_html();

    if let Some(lastest_chapter_fragment) = document.select(&chapter_selector).next() {
        let chapter_url = lastest_chapter_fragment
            .attr("href")
            .ok_or(UrasundayParseError::ChapterNotFound)?;
        let mut chapter_childrens = lastest_chapter_fragment.child_elements();
        let chapter_img = chapter_childrens
            .next()
            .ok_or(UrasundayParseError::ChapterNotFound)?
            .attr("src")
            .ok_or(UrasundayParseError::ChapterNotFound)?;
        let chapter_details = chapter_childrens
            .next()
            .ok_or(UrasundayParseError::ChapterNotFound)?;

        let mut chapter_details_children = chapter_details.child_elements();
        let chapter_title = {
            let first_title = chapter_details_children
                .next()
                .ok_or(UrasundayParseError::ChapterNotFound)?
                .inner_html();
            let second_title = chapter_details_children
                .next()
                .ok_or(UrasundayParseError::ChapterNotFound)?
                .inner_html();

            format!("{} {}", first_title, second_title)
        };

        let chapter_release_date = {
            let raw = chapter_details_children
                .next()
                .ok_or(UrasundayParseError::ChapterNotFound)?
                .inner_html();

            let naive_date = NaiveDate::parse_from_str(&raw, "%Y/%m/%d").unwrap();

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

    Err(UrasundayParseError::ChapterNotFound)
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
