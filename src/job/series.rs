use chrono::TimeZone;
use chrono_tz::Japan;
use sqlx::PgPool;

use crate::{
    core::{fetch::fetch_manga, manga::Manga},
    db::{
        inquiry::{get_manga_paginated, MangaQuery},
        model::MangaRow,
    },
};

#[derive(Debug)]
pub enum DiffingResult {
    NoChange,
    Upcoming(Manga),
    Released(Manga),
}

pub async fn update_books(pool: &PgPool) {
    // retrieve series from db (paginated) based on the current day
    // for each series check for latest update
    let mut page_counter = 1;
    let mut all_series: Vec<MangaRow> = Vec::new();

    loop {
        let series = get_manga_paginated(
            page_counter,
            10,
            MangaQuery {
                source: None,
                day: None,
            },
            pool,
        )
        .await;

        if let Ok(mut result) = series {
            if result.data.is_empty() {
                break;
            }

            all_series.append(&mut result.data);
        } else {
            panic!("Error retriving series from db {:?}", series);
        }

        page_counter += 1;
    }

    // generate diff state
    let mut tasks = vec![];

    for series in all_series {
        tasks.push(tokio::spawn(diff_update(series)));
    }

    for task in tasks {
        let task_result = task.await.unwrap();
    }

    // broadcast diff change to webhook and update database
    // TODO: add serenity crate for discord webhook
    // TODO: create query for db update

    todo!()
}

// TODO: use rate limiter when fetching update using governor
pub async fn diff_update(data: MangaRow) -> DiffingResult {
    // for each mangarow retrieve latest update
    let latest_update = fetch_manga(&data.manga_id, &data.source)
        .await
        .expect("Fail to fetch manga");

    // generate diffing result
    // no change -> title and release status doesn't change
    // upcoming -> release status change from released to not released and title change
    // released -> release status change from not released to released
    let current_dt = chrono::offset::Local::now().naive_local();
    let update_dt = latest_update.latest_chapter_release_date.naive_local();
    let released = Japan.from_local_datetime(&current_dt).unwrap()
        >= Japan.from_local_datetime(&update_dt).unwrap();

    if (data
        .latest_chapter_title
        .ne(&latest_update.latest_chapter_title)
        || !data.latest_chapter_released)
        && released
    {
        DiffingResult::Released(latest_update)
    } else if data
        .latest_chapter_title
        .ne(&latest_update.latest_chapter_title)
        && !released
    {
        DiffingResult::Upcoming(latest_update)
    } else {
        DiffingResult::NoChange
    }
}
