use chrono::{DateTime, Datelike, FixedOffset};
use chrono_tz::Japan;
use reqwest::Client;
use serde::Deserialize;
use serde_xml_rs::from_str;

use crate::core::{fetch::FetchError, types::Manga};

#[derive(Debug, Deserialize)]
pub struct Rss {
    pub channel: Channel,
}

#[derive(Debug, Deserialize)]
pub struct Channel {
    pub title: String,

    #[serde(rename = "pubDate")]
    #[serde(with = "rfc2822")]
    pub pub_date: DateTime<FixedOffset>,

    pub link: String,
    pub description: String,
    pub item: Vec<Item>,
}

#[cfg(feature = "ssr")]
impl TryFrom<Channel> for Manga {
    type Error = FetchError;

    fn try_from(value: Channel) -> Result<Self, Self::Error> {
        let latest_chapter = value
            .item
            .first()
            .ok_or(FetchError::ChapterNotFound(None))?;
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

#[derive(Debug, Deserialize)]
pub struct Item {
    pub title: String,
    pub link: String,
    pub guid: String,

    #[serde(rename = "pubDate")]
    #[serde(with = "rfc2822")]
    pub pub_date: DateTime<FixedOffset>,
    pub description: String,
    pub enclosure: Enclosure,
    pub author: String,
}

#[derive(Debug, Deserialize)]
pub struct Enclosure {
    pub url: String,
    pub length: String,
    pub r#type: String,
}

pub mod rfc2822 {
    use chrono::{DateTime, FixedOffset};
    use core::fmt;
    use serde::de;

    #[derive(Debug)]
    struct Rfc2822Visitor;

    /// Deserialize a [`DateTime`] from an RFC 2822 datetime
    ///
    /// Intended for use with `serde`s `deserialize_with` attribute.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(Rfc2822Visitor)
    }

    impl<'de> de::Visitor<'de> for Rfc2822Visitor {
        type Value = DateTime<FixedOffset>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an RFC 2822 formatted datetime string")
        }

        fn visit_str<E>(self, date_string: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            DateTime::parse_from_rfc2822(date_string).map_err(E::custom)
        }
    }
}

fn from_rss_xml(xml: &str) -> Result<Manga, FetchError> {
    let rss: Rss =
        from_str(xml).map_err(|e| FetchError::XmlDeserializeError(Some(e.to_string())))?;

    Manga::try_from(rss.channel)
}

pub async fn fetch_generic_rss(client: Client, url: String) -> Result<Manga, FetchError> {
    let rss = client
        .get(url)
        .send()
        .await
        .map_err(FetchError::ReqwestError)?
        .error_for_status()
        .map_err(FetchError::ReqwestError)?
        .text()
        .await
        .map_err(FetchError::ReqwestError)?;

    from_rss_xml(&rss)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde_xml_rs::from_str;

    use super::*;

    #[test]
    fn test_parse_generic_rss_source() {
        let paths = fs::read_dir("src/test_data/rss_manga").unwrap();

        for path in paths {
            let doc = fs::read_to_string(path.unwrap().path()).unwrap();
            let _: Rss = from_str(&doc).unwrap();
        }
    }
}
