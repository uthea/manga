use chrono::{DateTime, Datelike, FixedOffset, Weekday};
use chrono_tz::Japan;
use serde::{Deserialize, Serialize};

use super::parser::rss_manga::Channel;

#[derive(Deserialize, Serialize, Clone)]
pub struct Manga {
    pub title: String,
    pub cover_url: String,
    pub author: String,
    pub latest_chapter_title: String,
    pub latest_chapter_url: String,
    pub latest_chapter_release_date: DateTime<FixedOffset>,
    pub latest_chapter_publish_day: Weekday,
}

pub enum ConvertError {
    EmptyChapter,
}

impl TryFrom<Channel> for Manga {
    type Error = ConvertError;

    fn try_from(value: Channel) -> Result<Self, Self::Error> {
        let latest_chapter = value.item.first().ok_or(ConvertError::EmptyChapter)?;
        let release_date = &latest_chapter.pub_date.with_timezone(&Japan);

        Ok(Self {
            title: value.title,
            cover_url: latest_chapter.enclosure.url.clone(),
            author: latest_chapter.author.clone(),
            latest_chapter_title: latest_chapter.title.clone(),
            latest_chapter_url: latest_chapter.link.clone(),
            latest_chapter_release_date: latest_chapter.pub_date,
            latest_chapter_publish_day: release_date.weekday(),
        })
    }
}
