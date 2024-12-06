use crate::core::parser::rss_manga::ConvertError;
use serde_xml_rs::from_str;

use crate::core::parser::rss_manga::Rss;

use super::{
    parser::{
        comic_pixiv::{fetch_pixiv_data, PixivError},
        comic_walker::{fetch_comic_walker_data, ComicWalkerError},
        manga_up::{parse_manga_up_from_html, MangaUpError},
        urasunday::{parse_urasunday_from_html, UrasundayParseError},
        yanmaga::{parse_yanmaga_from_html, YanmagaParseError},
    },
    types::{Manga, MangaSource},
};

#[derive(Debug)]
pub enum FetchError {
    ConvertError(ConvertError),
    ReqwestError(reqwest::Error),
    DeserialzeXmlError(serde_xml_rs::Error),
    YanmagaParseError(YanmagaParseError),
    ComicPixivError(PixivError),
    UrasundayParseError(UrasundayParseError),
    ComicWalkerError(ComicWalkerError),
    MangaUpError(MangaUpError),
}

impl From<YanmagaParseError> for FetchError {
    fn from(value: YanmagaParseError) -> Self {
        Self::YanmagaParseError(value)
    }
}

impl From<PixivError> for FetchError {
    fn from(value: PixivError) -> Self {
        Self::ComicPixivError(value)
    }
}

impl From<UrasundayParseError> for FetchError {
    fn from(value: UrasundayParseError) -> Self {
        Self::UrasundayParseError(value)
    }
}

impl From<ComicWalkerError> for FetchError {
    fn from(value: ComicWalkerError) -> Self {
        Self::ComicWalkerError(value)
    }
}

impl From<MangaUpError> for FetchError {
    fn from(value: MangaUpError) -> Self {
        Self::MangaUpError(value)
    }
}

fn from_rss_xml(xml: &str) -> Result<Manga, FetchError> {
    let rss: Rss = from_str(xml).map_err(FetchError::DeserialzeXmlError)?;

    Manga::try_from(rss.channel).map_err(FetchError::ConvertError)
}

pub async fn fetch_manga(manga_id: &str, source: &MangaSource) -> Result<Manga, FetchError> {
    if source == &MangaSource::ComicPixiv {
        return fetch_pixiv_data(manga_id).await.map_err(FetchError::from);
    }

    if source == &MangaSource::ComicWalker {
        return fetch_comic_walker_data(manga_id)
            .await
            .map_err(FetchError::from);
    }

    let url = match source {
        MangaSource::ShounenJumpPlus => {
            format!("https://shonenjumpplus.com/rss/series/{}", manga_id)
        }
        MangaSource::ComicEarthStar => {
            format!("https://comic-earthstar.com/rss/series/{}", manga_id)
        }
        MangaSource::KurageBunch => format!("https://kuragebunch.com/rss/series/{}", manga_id),
        MangaSource::ComicGrowl => format!("https://comic-growl.com/rss/series/{}", manga_id),
        MangaSource::ComicDays => format!("https://comic-days.com/rss/series/{}", manga_id),
        MangaSource::MagazinePocket => {
            format!("https://pocket.shonenmagazine.com/rss/series/{}", manga_id)
        }
        MangaSource::Yanmaga => format!("https://yanmaga.jp/comics/{}", manga_id),
        MangaSource::ComicPixiv => unreachable!(),
        MangaSource::Urasunday => format!("https://urasunday.com/title/{}", manga_id),
        MangaSource::ComicWalker => unreachable!(),
        MangaSource::TonariYoungJump => format!("https://tonarinoyj.jp/rss/series/{}", manga_id),
        MangaSource::MangaUp => format!("https://www.manga-up.com/titles/{}", manga_id),
    };

    let response = reqwest::get(url)
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
        | MangaSource::ComicGrowl
        | MangaSource::ComicDays
        | MangaSource::MagazinePocket
        | MangaSource::TonariYoungJump => from_rss_xml(&response)?,

        MangaSource::Yanmaga => parse_yanmaga_from_html(response).map_err(FetchError::from)?,
        MangaSource::Urasunday => parse_urasunday_from_html(response).map_err(FetchError::from)?,
        MangaSource::ComicPixiv => unreachable!(),
        MangaSource::ComicWalker => unreachable!(),
        MangaSource::MangaUp => parse_manga_up_from_html(response).map_err(FetchError::from)?,
    };

    Ok(manga_info)
}
