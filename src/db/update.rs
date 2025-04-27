use sqlx::{PgPool, QueryBuilder};

use super::model::MangaRow;

pub async fn update_manga_batch(
    latest_data: impl Iterator<Item = &MangaRow>,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    let mut trx = pool.begin().await.expect("Error begin transaction");

    // create temp table
    sqlx::query(
        r#"
        create temp table update_table (
            source MangaSource not null, 
            manga_id text not null,
            title text not null,
            cover_url text not null,
            latest_chapter_title text not null,
            latest_chapter_url text not null,
            latest_chapter_release_date timestamp not null,
            latest_chapter_publish_day Weekday not null,
            latest_chapter_released boolean not null,
            last_update timestamp not null
        );
    "#,
    )
    .execute(&mut *trx)
    .await?;

    let mut query_builder = QueryBuilder::new(
        r#" 
        insert into update_table 
        (source, manga_id, title, cover_url, latest_chapter_title, latest_chapter_url, latest_chapter_release_date, latest_chapter_publish_day, latest_chapter_released, last_update) 
        "#,
    );

    query_builder.push_values(latest_data, |mut b, row| {
        b.push_bind(row.source.clone())
            .push_bind(row.manga_id.clone())
            .push_bind(row.title.clone())
            .push_bind(row.cover_url.clone())
            .push_bind(row.latest_chapter_title.clone())
            .push_bind(row.latest_chapter_url.clone())
            .push_bind(row.latest_chapter_release_date)
            .push_bind(row.latest_chapter_publish_day)
            .push_bind(row.latest_chapter_released)
            .push_bind(row.last_update);
    });

    query_builder.build().execute(&mut *trx).await?;

    let mut query_builder = QueryBuilder::new(
        r#"
         update series as s
            set
            title = u.title,
            cover_url = u.cover_url,
            latest_chapter_title = u.latest_chapter_title,
            latest_chapter_url = u.latest_chapter_url,
            latest_chapter_release_date = u.latest_chapter_release_date,
            latest_chapter_publish_day = u.latest_chapter_publish_day,
            latest_chapter_released = u.latest_chapter_released,
            last_update = u.last_update

        from update_table as u 
        where u.source = s.source and u.manga_id = s.manga_id
    "#,
    );

    query_builder.build().execute(&mut *trx).await?;

    sqlx::query(" DROP table update_table ")
        .execute(&mut *trx)
        .await?;

    trx.commit().await?;

    Ok(())
}
