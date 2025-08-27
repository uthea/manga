use crate::core::parser::{
    cdata_rss::fetch_cdata_rss, comic_fuz::fetch_comic_fuz, comic_pixiv::fetch_pixiv_data,
    comic_walker::fetch_comic_walker_data, gamma_plus::fetch_gamma_plus,
    gangan_online::fetch_gangan_online, ganma::fetch_ganma, manga_up::fetch_mangaup,
    mecha_comic::fetch_mecha_comic, rss_manga::fetch_generic_rss, urasunday::fetch_urasunday,
    yanmaga::fetch_yanmaga,
};
use fantoccini::error::{CmdError, NewSessionError};
use http::header;

use super::types::{Manga, MangaSource};

#[derive(Debug)]
pub enum FetchError {
    ReqwestError(reqwest::Error),
    JsonDeserializeError(serde_json::Error),
    XmlDeserializeError(Option<String>),
    ChapterNotFound(Option<String>),
    PageNotFound(Option<String>),
    WebDriverSessionError(NewSessionError),
    WebDriverCmdError(CmdError),
}

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36";

impl MangaSource {
    pub async fn fetch(&self, webdriver_url: &str, manga_id: &str) -> Result<Manga, FetchError> {
        let client = {
            let mut headers = header::HeaderMap::new();
            headers.insert("User-Agent", header::HeaderValue::from_static(USER_AGENT));
            reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .unwrap()
        };

        let manga = match self {
            MangaSource::Yanmaga => fetch_yanmaga(client, manga_id).await,
            MangaSource::ShounenJumpPlus => {
                fetch_generic_rss(
                    client,
                    format!("https://shonenjumpplus.com/rss/series/{manga_id}"),
                )
                .await
            }
            MangaSource::ComicEarthStar => {
                fetch_generic_rss(
                    client,
                    format!("https://comic-earthstar.com/rss/series/{manga_id}"),
                )
                .await
            }
            MangaSource::KurageBunch => {
                fetch_generic_rss(
                    client,
                    format!("https://kuragebunch.com/rss/series/{manga_id}"),
                )
                .await
            }
            MangaSource::ComicGrowl => {
                fetch_cdata_rss(
                    client,
                    format!("https://comic-growl.com/series/{manga_id}/rss"),
                )
                .await
            }
            MangaSource::ComicDays => {
                fetch_generic_rss(
                    client,
                    format!("https://comic-days.com/rss/series/{manga_id}"),
                )
                .await
            }
            MangaSource::MagazinePocket => {
                fetch_generic_rss(
                    client,
                    format!("https://mgpk-cdn.magazinepocket.com/static/rss/{manga_id}/feed.xml"),
                )
                .await
            }
            MangaSource::TonariYoungJump => {
                fetch_generic_rss(
                    client,
                    format!("https://tonarinoyj.jp/rss/series/{manga_id}"),
                )
                .await
            }
            MangaSource::SundayWebry => {
                fetch_generic_rss(
                    client,
                    format!("https://www.sunday-webry.com/rss/series/{manga_id}"),
                )
                .await
            }
            MangaSource::ComicAction => {
                fetch_generic_rss(
                    client,
                    format!("https://comic-action.com/rss/series/{manga_id}"),
                )
                .await
            }
            MangaSource::ComicGardo => {
                fetch_generic_rss(
                    client,
                    format!("https://comic-gardo.com/rss/series/{manga_id}"),
                )
                .await
            }

            MangaSource::ComicPixiv => fetch_pixiv_data(client, manga_id).await,
            MangaSource::Urasunday => fetch_urasunday(webdriver_url, manga_id).await,
            MangaSource::ComicWalker => fetch_comic_walker_data(client, manga_id).await,
            MangaSource::MangaUp => fetch_mangaup(client, manga_id).await,
            MangaSource::ComicFuz => fetch_comic_fuz(client, manga_id).await,
            MangaSource::GanganOnline => fetch_gangan_online(client, manga_id).await,
            MangaSource::GammaPlus => fetch_gamma_plus(client, manga_id).await,
            MangaSource::ChampionCross => {
                fetch_cdata_rss(
                    client,
                    format!("https://championcross.jp/series/{manga_id}/rss"),
                )
                .await
            }
            MangaSource::GANMA => fetch_ganma(client, manga_id).await,
            MangaSource::YoungAnimal => {
                fetch_cdata_rss(
                    client,
                    format!("https://younganimal.com/series/{manga_id}/rss"),
                )
                .await
            }
            MangaSource::MechaComic => fetch_mecha_comic(client, manga_id).await,
            MangaSource::YoungChampion => {
                fetch_cdata_rss(
                    client,
                    format!("https://youngchampion.jp/series/{manga_id}/rss"),
                )
                .await
            }
            MangaSource::IchijinPlus => {
                fetch_generic_rss(
                    client,
                    format!("https://ichicomi.com/rss/series/{manga_id}"),
                )
                .await
            }
        }?;

        Ok(self.postprocess(manga))
    }

    fn postprocess(&self, mut manga: Manga) -> Manga {
        manga.title = self.cleanup_title(&manga.title);
        manga.latest_chapter_url = self.replace_episode_url(&manga.latest_chapter_url);
        manga
    }

    pub fn cleanup_title(&self, title: &str) -> String {
        let mut removed_suffix = match self {
            MangaSource::ShounenJumpPlus => title.replace("少年ジャンプ＋", "").trim().to_owned(),
            MangaSource::ComicEarthStar => title.replace("コミック アース・スター｜毎週木曜・最新話更新！無料で漫画が読めるWEBコミック誌", "").trim().to_owned(),
            MangaSource::KurageBunch => title.replace("くらげバンチ", "").trim().to_owned(),
            MangaSource::ComicDays => title.replace("コミックDAYS", "").trim().to_owned(),
            MangaSource::MagazinePocket => title.replace("マガポケ", "").trim().to_owned(),
            MangaSource::TonariYoungJump => title.replace("となりのヤングジャンプ", "").trim().to_owned(),
            MangaSource::SundayWebry => title.replace("サンデーうぇぶり", "").trim().to_owned(),
            MangaSource::ComicAction => title.replace("webアクション｜双葉社発のマンガサイト", "").trim().to_owned(),
            MangaSource::IchijinPlus => title.replace("一迅プラス", "").trim().to_owned(),
            MangaSource::ComicGardo => title.replace("コミックガルド", "").trim().to_owned(),
            _ => title.to_owned()
,
        };

        match self {
            MangaSource::ShounenJumpPlus
            | MangaSource::ComicEarthStar
            | MangaSource::KurageBunch
            | MangaSource::ComicDays
            | MangaSource::MagazinePocket
            | MangaSource::TonariYoungJump
            | MangaSource::SundayWebry
            | MangaSource::IchijinPlus
            | MangaSource::ComicAction
            | MangaSource::ComicGardo => {
                removed_suffix.pop();
                removed_suffix.remove(0);
            }
            _ => {}
        };

        removed_suffix
    }

    pub fn replace_episode_url(&self, url: &str) -> String {
        // for now this is for megazine pocket since the new url is not working yet
        match self {
            MangaSource::MagazinePocket => url
                .replace("mgpk-web.magazinepocket.com", "pocket.shonenmagazine.com")
                .to_owned(),
            _ => url.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_cleanup_shounen_jump_plus() {
        let title = "少年ジャンプ＋（魔都精兵のスレイブ）";
        let expected = "魔都精兵のスレイブ";

        let source = MangaSource::ShounenJumpPlus;
        let got = source.cleanup_title(title);
        assert_eq!(got, expected);
    }

    #[test]
    fn test_cleanup_comic_earth_star() {
        let title = "コミック アース・スター｜毎週木曜・最新話更新！無料で漫画が読めるWEBコミック誌（願いを叶えてもらおうと悪魔を召喚したけど、可愛かったので結婚しました　～悪魔の新妻～）";
        let expected =
            "願いを叶えてもらおうと悪魔を召喚したけど、可愛かったので結婚しました　～悪魔の新妻～";

        let source = MangaSource::ComicEarthStar;
        let got = source.cleanup_title(title);
        assert_eq!(got, expected);
    }

    #[test]
    fn test_cleanup_kurage_bunch() {
        let title = "くらげバンチ（三咲くんは攻略キャラじゃない）";
        let expected = "三咲くんは攻略キャラじゃない";

        let source = MangaSource::KurageBunch;
        let got = source.cleanup_title(title);
        assert_eq!(got, expected);
    }

    #[test]
    fn test_cleanup_comic_days() {
        let title = "コミックDAYS（外れスキル《木の実マスター》　～スキルの実（食べたら死ぬ）を無限に食べられるようになった件について～）";
        let expected = "外れスキル《木の実マスター》　～スキルの実（食べたら死ぬ）を無限に食べられるようになった件について～";

        let source = MangaSource::ComicDays;
        let got = source.cleanup_title(title);
        assert_eq!(got, expected);
    }

    #[test]
    fn test_cleanup_magazine_pocket() {
        let title =
            "マガポケ（不遇職【鑑定士】が実は最強だった～奈落で鍛えた最強の【神眼】で無双する～）";
        let expected = "不遇職【鑑定士】が実は最強だった～奈落で鍛えた最強の【神眼】で無双する～";

        let source = MangaSource::MagazinePocket;
        let got = source.cleanup_title(title);
        assert_eq!(got, expected);
    }

    #[test]
    fn test_replace_episode_url_magazine_pocket() {
        let episode_url = "https://mgpk-web.magazinepocket.com/title/02174/episode/381450";
        let expected = "https://pocket.shonenmagazine.com/title/02174/episode/381450";
        let source = MangaSource::MagazinePocket;
        let got = source.replace_episode_url(episode_url);
        assert_eq!(got, expected);
    }

    #[test]
    fn test_cleanup_tonari_young_jump() {
        let title = "となりのヤングジャンプ（つれないほど青くて あざといくらいに赤い）";
        let expected = "つれないほど青くて あざといくらいに赤い";

        let source = MangaSource::TonariYoungJump;
        let got = source.cleanup_title(title);
        assert_eq!(got, expected);
    }

    #[test]
    fn test_cleanup_sunday_webry() {
        let title = "サンデーうぇぶり（となりの席のヤツがそういう目で見てくる）";
        let expected = "となりの席のヤツがそういう目で見てくる";

        let source = MangaSource::SundayWebry;
        let got = source.cleanup_title(title);
        assert_eq!(got, expected);
    }

    #[test]
    fn test_cleanup_comic_action() {
        let title = "webアクション｜双葉社発のマンガサイト（かくして！マキナさん!!）";
        let expected = "かくして！マキナさん!!";

        let source = MangaSource::ComicAction;
        let got = source.cleanup_title(title);
        assert_eq!(got, expected);
    }

    #[test]
    fn test_cleanup_ichijin_plus() {
        let title = "一迅プラス（映しちゃダメな顔）";
        let expected = "映しちゃダメな顔";

        let source = MangaSource::IchijinPlus;
        let got = source.cleanup_title(title);
        assert_eq!(got, expected);
    }

    #[test]
    fn test_cleanup_comic_gardo() {
        let title = "コミックガルド（××しないで！月峰さん。）";
        let expected = "××しないで！月峰さん。";

        let source = MangaSource::ComicGardo;
        let got = source.cleanup_title(title);
        assert_eq!(got, expected);
    }

    #[test]
    fn test_cleanup_do_nothing() {
        let title = "恋する(おとめ)の作り方";

        let source = MangaSource::ComicPixiv;
        let got = source.cleanup_title(title);
        assert_eq!(got, title);
    }
}
