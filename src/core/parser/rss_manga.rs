use chrono::{DateTime, FixedOffset};
use serde::Deserialize;

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
