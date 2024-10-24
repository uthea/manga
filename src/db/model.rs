use chrono::{NaiveDateTime, Weekday};

use crate::core::types::MangaSource;

#[derive(sqlx::FromRow, Debug)]
pub struct MangaRow {
    pub source: MangaSource,
    pub manga_id: String,
    pub cover_url: String,
    pub author: String,
    pub title: String,
    pub latest_chapter_title: String,
    pub latest_chapter_url: String,
    pub latest_chapter_release_date: NaiveDateTime,
    pub latest_chapter_publish_day: DbWeekday,
    pub latest_chapter_released: bool,
    pub last_update: NaiveDateTime,
}

#[derive(sqlx::Type, Debug)]
#[sqlx(type_name = "Weekday")]
pub enum DbWeekday {
    Mon = 0,
    Tue = 1,
    Wed = 2,
    Thu = 3,
    Fri = 4,
    Sat = 5,
    Sun = 6,
}

impl From<Weekday> for DbWeekday {
    fn from(value: Weekday) -> Self {
        match value {
            Weekday::Mon => DbWeekday::Mon,
            Weekday::Tue => DbWeekday::Tue,
            Weekday::Wed => DbWeekday::Wed,
            Weekday::Thu => DbWeekday::Thu,
            Weekday::Fri => DbWeekday::Fri,
            Weekday::Sat => DbWeekday::Sat,
            Weekday::Sun => DbWeekday::Sun,
        }
    }
}

impl From<DbWeekday> for Weekday {
    fn from(value: DbWeekday) -> Self {
        match value {
            DbWeekday::Mon => Weekday::Mon,
            DbWeekday::Tue => Weekday::Tue,
            DbWeekday::Wed => Weekday::Wed,
            DbWeekday::Thu => Weekday::Thu,
            DbWeekday::Fri => Weekday::Fri,
            DbWeekday::Sat => Weekday::Sat,
            DbWeekday::Sun => Weekday::Sun,
        }
    }
}
