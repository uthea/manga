use sqlx::{PgPool, QueryBuilder};

use crate::core::types::MangaSource;

pub async fn delete_manga_bulk<I>(manga_list: I, pool: &PgPool) -> Result<u64, sqlx::Error>
where
    I: IntoIterator<Item = (MangaSource, String)>,
{
    let mut query_builder = QueryBuilder::new("delete from series where (source, manga_id) in ");
    query_builder.push_tuples(manga_list, |mut b, (source, id)| {
        b.push_bind(source);
        b.push_bind(id);
    });

    let query = query_builder.build();

    let query_result = query.execute(pool).await?;

    Ok(query_result.rows_affected())
}
