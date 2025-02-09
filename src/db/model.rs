use crate::core::{types::Manga, types::MangaSource};
use chrono::TimeZone;
use chrono::{Local, NaiveDateTime, Weekday};
use chrono_tz::Japan;

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

impl MangaRow {
    pub fn from_manga(manga_id: String, source: MangaSource, info: Manga) -> Self {
        let current_dt = chrono::offset::Utc::now();
        let release_dt = info.latest_chapter_release_date.naive_local();
        let wd: DbWeekday = info.latest_chapter_publish_day.into();

        Self {
            source,
            manga_id,
            cover_url: info.cover_url,
            author: info.author,
            title: info.title,
            latest_chapter_title: info.latest_chapter_title,
            latest_chapter_url: info.latest_chapter_url,
            latest_chapter_release_date: release_dt,
            latest_chapter_publish_day: wd,
            latest_chapter_released: current_dt.with_timezone(&Japan)
                >= Japan.from_local_datetime(&release_dt).unwrap(),
            last_update: chrono::offset::Local::now().naive_local(),
        }
    }

    pub fn into_manga(self) -> Manga {
        Manga {
            title: self.title,
            cover_url: self.cover_url,
            author: self.author,
            latest_chapter_title: self.latest_chapter_title,
            latest_chapter_url: self.latest_chapter_url,
            latest_chapter_release_date: Local
                .from_local_datetime(&self.latest_chapter_release_date)
                .single()
                .unwrap()
                .fixed_offset(),
            latest_chapter_publish_day: self.latest_chapter_publish_day.into(),
        }
    }
}

#[derive(sqlx::Type, Debug, Copy, Clone)]
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
