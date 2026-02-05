use chrono::{DateTime, Datelike, FixedOffset};
use chrono_tz::Japan;
use reqwest::Client;
use xmlserde::{xml_deserialize_from_str, XmlValue};
use xmlserde_derives::XmlDeserialize;

use crate::core::{fetch::FetchError, types::Manga};

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
    type Error = FetchError;

    fn try_from(value: Channel) -> Result<Self, Self::Error> {
        let latest_chapter = value
            .item
            .first()
            .ok_or(FetchError::ChapterNotFound(None))?;
        let release_date = &latest_chapter.pub_date.date.0.with_timezone(&Japan);

        let thumbnail = {
            let t = latest_chapter.thumbnail.inner.clone();
            let u = latest_chapter.thumbnail.url.clone();

            let mut res = "".into();

            if let Some(val) = t {
                res = val;
            } else if let Some(url) = u {
                res = url;
            }

            res
        };

        Ok(Self {
            title: value.title.inner.trim().to_owned(),
            cover_url: thumbnail,
            author: latest_chapter
                .creator
                .inner
                .clone()
                .unwrap_or("".to_owned())
                .clone(),
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

    #[xmlserde(name = b"pubDate", ty = "child")]
    pub pub_date: DateValue,

    #[xmlserde(name = b"media:thumbnail", ty = "child")]
    pub thumbnail: ThumbnailValue,

    // no author related information in the rss (in case of gamma plus)
    #[xmlserde(name = b"dc:creator", ty = "child")]
    pub creator: OptionalValue,
}

#[derive(Debug, XmlDeserialize, Clone)]
pub struct Value {
    #[xmlserde(ty = "text")]
    pub inner: String,
}

#[derive(Debug, XmlDeserialize, Clone)]
pub struct OptionalValue {
    #[xmlserde(ty = "text")]
    pub inner: Option<String>,
}

#[derive(Debug, XmlDeserialize)]
pub struct ThumbnailValue {
    #[xmlserde(ty = "text")]
    pub inner: Option<String>,

    #[xmlserde(name = b"url", ty = "attr")] // gamma plus use url tag
    pub url: Option<String>,
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

pub fn parse_cdata_xml(xml: String) -> Result<Manga, FetchError> {
    let result: Rss = xml_deserialize_from_str(&xml.replace("<![CDATA[", "").replace("]]>", ""))
        .map_err(|e| FetchError::XmlDeserializeError(Some(e)))?;

    Manga::try_from(result.channel)
}

pub async fn fetch_cdata_rss(client: Client, url: String) -> Result<Manga, FetchError> {
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

    parse_cdata_xml(rss)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_parse_cdata_rss() {
        let paths = fs::read_dir("src/test_data/cdata_rss").unwrap();

        for path in paths {
            let doc = fs::read_to_string(path.unwrap().path()).unwrap();

            let result = parse_cdata_xml(doc)
                .inspect_err(|e| {
                    dbg!(e);
                })
                .unwrap();

            dbg!(result);
        }
    }
}
