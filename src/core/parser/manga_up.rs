use crate::core::types::Manga;
use chrono::{Datelike, Local};
use regex::{Regex, RegexBuilder};
use scraper::{Html, Selector};
use serde::Deserialize;
use std::sync::LazyLock;

static CHAPTER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(r#"\{\\"titleName\\.*currentChapter.*\],\[\\"\$\\",\\"\$L6f"#)
        .multi_line(true)
        .build()
        .unwrap()
});

#[derive(Debug)]
pub enum MangaUpError {
    TitleNotFound,
    AuthorNotFound,
    ChapterNotFound,
}

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

pub fn parse_manga_up_from_html(html: String) -> Result<Manga, MangaUpError> {
    let title_selector = Selector::parse(r#"h2[class*="pc:text-title-lg-pc"]"#).unwrap();
    let author_selector = Selector::parse(
        r#"div[class="text-on_background_medium sp:text-body-md-sp pc:text-body-md-pc"]"#,
    )
    .unwrap();

    let document = Html::parse_document(&html);
    let title = document
        .select(&title_selector)
        .next()
        .ok_or(MangaUpError::TitleNotFound)?
        .inner_html();
    let author = document
        .select(&author_selector)
        .next()
        .ok_or(MangaUpError::AuthorNotFound)?
        .inner_html();

    // parse chapter related data
    let captures = CHAPTER_REGEX
        .captures(&html)
        .ok_or(MangaUpError::ChapterNotFound)?;

    let chapter_data_json = captures
        .get(0)
        .ok_or(MangaUpError::ChapterNotFound)?
        .as_str()
        .replace(r#"],[\"$\",\"$L6f"#, "")
        .replace("\\", "");

    let chapter_data: MangaUpData =
        serde_json::from_str(&chapter_data_json).map_err(|_| MangaUpError::ChapterNotFound)?;

    let latest_chapter = chapter_data
        .chapters
        .last()
        .ok_or(MangaUpError::ChapterNotFound)?;

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
