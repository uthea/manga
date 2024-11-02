use sqlx::PgPool;

use crate::{
    core::types::MangaSource,
    db::inquiry::{get_manga_paginated, MangaQuery},
};

pub async fn update_books(pool: &PgPool) {
    // retrieve series from db (paginated) based on the current day
    // for each series check for latest update
    let page_counter = 1;
    let series = get_manga_paginated(
        page_counter,
        10,
        MangaQuery {
            source: Some(MangaSource::ShounenJumpPlus),
            day: None,
        },
        pool,
    )
    .await
    .unwrap();

    dbg!(series);

    // not released -> released and title doesn't change -> mark as released
    // released -> not released and title change -> update metadata and mark as upcoming
    // released -> released and title change -> mark as released and update metadata
}
