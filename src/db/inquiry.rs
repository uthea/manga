use sqlx::PgPool;

use crate::core::types::MangaSource;

use super::model::MangaRow;

pub async fn get_manga(
    source: &MangaSource,
    manga_id: &str,
    pool: &PgPool,
) -> Result<MangaRow, sqlx::Error> {
    let row =
        sqlx::query_as::<_, MangaRow>("select * from series where source = $1 and manga_id = $2")
            .bind(source)
            .bind(manga_id)
            .fetch_one(pool)
            .await?;

    Ok(row)
}
