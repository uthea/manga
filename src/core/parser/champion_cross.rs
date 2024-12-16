use chrono::{DateTime, Datelike, FixedOffset};
use chrono_tz::Japan;
use xmlserde::{xml_deserialize_from_str, XmlValue};
use xmlserde_derives::XmlDeserialize;

use crate::core::types::Manga;

#[derive(Debug)]
pub enum ChampionCrossError {
    DeserializeError,
    EmptyChapter,
}

#[derive(Debug, XmlDeserialize)]
#[xmlserde(root = b"rss")]
pub struct Rss {
    #[xmlserde(name = b"channel", ty = "child")]
    pub channel: Channel,
}

#[derive(Debug, XmlDeserialize)]
pub struct Channel {
    #[xmlserde(name = b"title", ty = "child")]
    pub title: Value,

    #[xmlserde(name = b"item", ty = "child")]
    pub item: Vec<Item>,
}

impl TryFrom<Channel> for Manga {
    type Error = ChampionCrossError;

    fn try_from(value: Channel) -> Result<Self, Self::Error> {
        let latest_chapter = value.item.first().ok_or(ChampionCrossError::EmptyChapter)?;
        let release_date = &latest_chapter.pub_date.date.0.with_timezone(&Japan);

        Ok(Self {
            title: value.title.inner.trim().to_owned(),
            cover_url: latest_chapter.thumbnail.inner.clone(),
            author: latest_chapter.creator.inner.clone(),
            latest_chapter_title: latest_chapter.title.inner.trim().to_owned(),
            latest_chapter_url: latest_chapter.link.inner.clone(),
            latest_chapter_release_date: latest_chapter.pub_date.date.0,
            latest_chapter_publish_day: release_date.weekday(),
        })
    }
}

#[derive(Debug, XmlDeserialize)]
pub struct Item {
    #[xmlserde(name = b"title", ty = "child")]
    pub title: Value,

    #[xmlserde(name = b"link", ty = "child")]
    pub link: Value,

    #[xmlserde(name = b"guid", ty = "child")]
    pub guid: Value,

    #[xmlserde(name = b"pubDate", ty = "child")]
    pub pub_date: DateValue,

    #[xmlserde(name = b"media:thumbnail", ty = "child")]
    pub thumbnail: Value,

    #[xmlserde(name = b"dc:creator", ty = "child")]
    pub creator: Value,
}

#[derive(Debug, XmlDeserialize)]
pub struct Value {
    #[xmlserde(ty = "text")]
    pub inner: String,
}

#[derive(Debug, XmlDeserialize)]
pub struct DateValue {
    #[xmlserde(ty = "text")]
    pub date: Rfc2822Date,
}

#[derive(Debug)]
pub struct Rfc2822Date(pub DateTime<FixedOffset>);

impl XmlValue for Rfc2822Date {
    fn serialize(&self) -> String {
        self.0.to_string()
    }

    fn deserialize(s: &str) -> Result<Self, String> {
        DateTime::parse_from_rfc2822(s)
            .map_err(|e| e.to_string())
            .map(Rfc2822Date)
    }
}

pub fn parse_champion_cross_xml(xml: String) -> Result<Manga, ChampionCrossError> {
    let result: Rss = xml_deserialize_from_str(&xml.replace("<![CDATA[", "").replace("]]>", ""))
        .map_err(|_| ChampionCrossError::DeserializeError)?;

    Manga::try_from(result.channel)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_parse_champion_cross_rss() {
        let paths = fs::read_dir("src/test_data/champion_cross").unwrap();

        for path in paths {
            let doc = fs::read_to_string(path.unwrap().path()).unwrap();

            parse_champion_cross_xml(doc).unwrap();
        }
    }
}
