use crate::core::parser::{
    cdata_rss::fetch_cdata_rss, comic_fuz::fetch_comic_fuz, comic_pixiv::fetch_pixiv_data,
    comic_walker::fetch_comic_walker_data, gamma_plus::fetch_gamma_plus,
    gangan_online::fetch_gangan_online, ganma::fetch_ganma, manga_up::fetch_mangaup,
    rss_manga::fetch_generic_rss, urasunday::fetch_urasunday, yanmaga::fetch_yanmaga,
};
use http::header;

use super::types::{Manga, MangaSource};

#[derive(Debug)]
pub enum FetchError {
    ReqwestError(reqwest::Error),
    JsonDeserializeError(serde_json::Error),
    XmlDeserializeError(Option<String>),
    ChapterNotFound(Option<String>),
    PageNotFound(Option<String>),
}

impl MangaSource {
    pub async fn fetch(&self, manga_id: &str) -> Result<Manga, FetchError> {
        let client = {
            let mut headers = header::HeaderMap::new();
            headers.insert("User-Agent", header::HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36"));
            reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .unwrap()
        };

        match self {
            MangaSource::Yanmaga => fetch_yanmaga(client, manga_id).await,
            MangaSource::ShounenJumpPlus => {
                fetch_generic_rss(
                    client,
                    format!("https://shonenjumpplus.com/rss/series/{}", manga_id),
                )
                .await
            }
            MangaSource::ComicEarthStar => {
                fetch_generic_rss(
                    client,
                    format!("https://comic-earthstar.com/rss/series/{}", manga_id),
                )
                .await
            }
            MangaSource::KurageBunch => {
                fetch_generic_rss(
                    client,
                    format!("https://kuragebunch.com/rss/series/{}", manga_id),
                )
                .await
            }
            MangaSource::ComicGrowl => {
                fetch_generic_rss(
                    client,
                    format!("https://comic-growl.com/rss/series/{}", manga_id),
                )
                .await
            }
            MangaSource::ComicDays => {
                fetch_generic_rss(
                    client,
                    format!("https://comic-days.com/rss/series/{}", manga_id),
                )
                .await
            }
            MangaSource::MagazinePocket => {
                fetch_generic_rss(
                    client,
                    format!("https://pocket.shonenmagazine.com/rss/series/{}", manga_id),
                )
                .await
            }
            MangaSource::TonariYoungJump => {
                fetch_generic_rss(
                    client,
                    format!("https://tonarinoyj.jp/rss/series/{}", manga_id),
                )
                .await
            }
            MangaSource::SundayWebry => {
                fetch_generic_rss(
                    client,
                    format!("https://www.sunday-webry.com/rss/series/{}", manga_id),
                )
                .await
            }

            MangaSource::ComicPixiv => fetch_pixiv_data(client, manga_id).await,
            MangaSource::Urasunday => fetch_urasunday(client, manga_id).await,
            MangaSource::ComicWalker => fetch_comic_walker_data(client, manga_id).await,
            MangaSource::MangaUp => fetch_mangaup(client, manga_id).await,
            MangaSource::ComicFuz => fetch_comic_fuz(client, manga_id).await,
            MangaSource::GanganOnline => fetch_gangan_online(client, manga_id).await,
            MangaSource::GammaPlus => fetch_gamma_plus(client, manga_id).await,
            MangaSource::ChampionCross => {
                fetch_cdata_rss(
                    client,
                    format!("https://championcross.jp/series/{}/rss", manga_id),
                )
                .await
            }
            MangaSource::GANMA => fetch_ganma(client, manga_id).await,
            MangaSource::YoungAnimal => {
                fetch_cdata_rss(
                    client,
                    format!("https://younganimal.com/series/{}/rss", manga_id),
                )
                .await
            }
        }
    }
}
