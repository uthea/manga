use chrono::TimeZone;
use chrono_tz::Japan;
use sqlx::PgPool;

use crate::core::{manga::Manga, types::MangaSource};

use super::model::DbWeekday;

pub async fn insert_manga(
    source: MangaSource,
    manga_id: String,
    info: Manga,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    let current_dt = chrono::offset::Local::now().naive_local();
    let release_dt = info.latest_chapter_release_date.naive_local();
    let wd: DbWeekday = info.latest_chapter_publish_day.into();

    sqlx::query(r#"
        INSERT INTO series
        ("source", manga_id, title, cover_url, author, latest_chapter_title, 
        latest_chapter_url, latest_chapter_release_date, latest_chapter_publish_day, latest_chapter_released, last_update)
        VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
    "#)
        .bind(source)
        .bind(manga_id)
        .bind(info.title)
        .bind(info.cover_url)
        .bind(info.author)
        .bind(info.latest_chapter_title)
        .bind(info.latest_chapter_url)
        .bind(release_dt)
        .bind(wd)
        .bind(Japan.from_local_datetime(&current_dt).unwrap() >= Japan.from_local_datetime(&release_dt).unwrap())
        .bind(current_dt)
        .execute(pool)
        .await?;

    Ok(())
}
