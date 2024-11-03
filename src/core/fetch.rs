use serde_xml_rs::from_str;

use crate::core::parser::rss_manga::Rss;

use super::{
    manga::{ConvertError, Manga},
    types::MangaSource,
};

#[derive(Debug)]
pub enum FetchError {
    ConvertError(ConvertError),
    ReqwestError(reqwest::Error),
    DeserialzeXmlError(serde_xml_rs::Error),
}

fn from_rss_xml(xml: &str) -> Result<Manga, FetchError> {
    let rss: Rss = from_str(xml).map_err(FetchError::DeserialzeXmlError)?;

    Manga::try_from(rss.channel).map_err(FetchError::ConvertError)
}

pub async fn fetch_manga(manga_id: &str, source: &MangaSource) -> Result<Manga, FetchError> {
    let base_url = match source {
        MangaSource::ShounenJumpPlus => "https://shonenjumpplus.com/rss/series/",
        MangaSource::ComicEarthStar => "https://comic-earthstar.com/rss/series/",
        MangaSource::KurageBunch => "https://kuragebunch.com/rss/series/",
        MangaSource::ComicGrowl => "https://comic-growl.com/rss/series/",
    };

    let response = reqwest::get(format!("{}{}", base_url, manga_id))
        .await
        .map_err(FetchError::ReqwestError)?
        .error_for_status()
        .map_err(FetchError::ReqwestError)?
        .text()
        .await
        .map_err(FetchError::ReqwestError)?;

    let manga_info = match source {
        MangaSource::ShounenJumpPlus
        | MangaSource::ComicEarthStar
        | MangaSource::KurageBunch
        | MangaSource::ComicGrowl => from_rss_xml(&response)?,
    };

    Ok(manga_info)
}
