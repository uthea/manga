use chrono::{DateTime, FixedOffset, Weekday};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Default, Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct MangaQuery {
    pub source: Option<MangaSource>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub chapter_title: Option<String>,
    pub day: Option<Weekday>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Paginated<T> {
    pub data: T,
    pub total_page: i64,
}

#[derive(
    EnumIter,
    serde::Deserialize,
    serde::Serialize,
    Clone,
    Debug,
    EnumString,
    Display,
    Eq,
    PartialEq,
    Hash,
)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
#[cfg_attr(feature = "ssr", sqlx(type_name = "MangaSource"))]
pub enum MangaSource {
    #[strum(to_string = "YanMaga")]
    Yanmaga,

    #[strum(to_string = "Shounen Jump Plus")]
    ShounenJumpPlus,

    #[strum(to_string = "Comic Earthstar")]
    ComicEarthStar,

    #[strum(to_string = "Kurage Bunch")]
    KurageBunch,

    #[strum(to_string = "Comic Growl")]
    ComicGrowl,

    #[strum(to_string = "Comic Days")]
    ComicDays,

    #[strum(to_string = "Magazine Pocket")]
    MagazinePocket,

    #[strum(to_string = "Comic Pixiv")]
    ComicPixiv,

    #[strum(to_string = "Urasunday")]
    Urasunday,

    #[strum(to_string = "Comic Walker")]
    ComicWalker,

    #[strum(to_string = "Tonari Young Jump")]
    TonariYoungJump,

    #[strum(to_string = "Manga Up")]
    MangaUp,

    #[strum(to_string = "Sunday Webry")]
    SundayWebry,

    #[strum(to_string = "Comic Fuz")]
    ComicFuz,

    #[strum(to_string = "Gangan Online")]
    GanganOnline,

    #[strum(to_string = "Gamma Plus")]
    GammaPlus,

    #[strum(to_string = "Champion Cross")]
    ChampionCross,

    #[strum(to_string = "GANMA")]
    GANMA,

    #[strum(to_string = "Young Animal")]
    YoungAnimal,

    #[strum(to_string = "Mecha Comic")]
    MechaComic,

    #[strum(to_string = "Young Champion")]
    YoungChampion,

    #[strum(to_string = "Ichijin Plus")]
    IchijinPlus,

    #[strum(to_string = "Comic Action")]
    ComicAction,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Manga {
    pub title: String,
    pub cover_url: String,
    pub author: String,
    pub latest_chapter_title: String,
    pub latest_chapter_url: String,
    pub latest_chapter_release_date: DateTime<FixedOffset>,
    pub latest_chapter_publish_day: Weekday,
}
