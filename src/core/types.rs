use strum_macros::{Display, EnumIter, EnumString};

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
    // #[strum(to_string = "Yan Maga")]
    // Yanmaga,
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
}
