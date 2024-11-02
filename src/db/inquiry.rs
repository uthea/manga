use chrono::Weekday;
use sqlx::{FromRow, PgPool, QueryBuilder, Row};

use crate::core::types::MangaSource;

use super::model::{DbWeekday, MangaRow, Paginated};

#[derive(Default)]
pub struct MangaQuery {
    pub source: Option<MangaSource>,
    pub day: Option<Weekday>,
}

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

pub async fn get_manga_paginated(
    page_number: i64,
    page_size: i64,
    query_option: MangaQuery,
    pool: &PgPool,
) -> Result<Paginated<Vec<MangaRow>>, sqlx::Error> {
    let mut query = QueryBuilder::new(
        r#"
        with cte AS (
            select * from series 
            where 1=1
    "#,
    );

    if let Some(source) = &query_option.source {
        query.push(" AND source =  ");
        query.push_bind(source);
    }

    if let Some(day) = &query_option.day {
        query.push(" AND latest_chapter_publish_day =  ");
        query.push_bind(DbWeekday::from(*day));
    }

    query.push(" ) select *, count(*) over () as total_count from cte ORDER BY manga_id LIMIT ");
    query.push_bind(page_size);
    query.push(" OFFSET ");
    query.push_bind(if page_number == 1 {
        0
    } else {
        page_number * page_size
    });

    let result = query.build().fetch_all(pool).await?;

    let total_rows: i64 = if let Some(row) = result.first() {
        row.try_get("total_count").unwrap()
    } else {
        0
    };

    let data: Vec<MangaRow> = result
        .iter()
        .map(|row| MangaRow::from_row(row).unwrap())
        .collect();

    Ok(Paginated {
        data,
        total_page: if total_rows == 0 {
            0
        } else if total_rows < page_size {
            1
        } else {
            total_rows / page_size
        },
    })
}
