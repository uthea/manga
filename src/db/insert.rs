use sqlx::PgPool;

use crate::core::{manga::Manga, types::MangaSource};

use super::model::MangaRow;

pub async fn insert_manga(
    source: MangaSource,
    manga_id: String,
    info: Manga,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    let manga_row = MangaRow::from_manga(manga_id, source, info);

    sqlx::query(r#"
        INSERT INTO series
        ("source", manga_id, title, cover_url, author, latest_chapter_title, 
        latest_chapter_url, latest_chapter_release_date, latest_chapter_publish_day, latest_chapter_released, last_update)
        VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
    "#)
        .bind(manga_row.source)
        .bind(manga_row.manga_id)
        .bind(manga_row.title)
        .bind(manga_row.cover_url)
        .bind(manga_row.author)
        .bind(manga_row.latest_chapter_title)
        .bind(manga_row.latest_chapter_url)
        .bind(manga_row.latest_chapter_release_date)
        .bind(manga_row.latest_chapter_publish_day)
        .bind(manga_row.latest_chapter_released)
        .bind(manga_row.last_update)
        .execute(pool)
        .await?;

    Ok(())
}
