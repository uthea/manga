use std::{num::NonZeroU32, sync::Arc, time::Duration};

use governor::{DefaultKeyedRateLimiter, Jitter, Quota, RateLimiter};
use serenity::all::{CreateEmbed, ExecuteWebhook, Http, Webhook};
use sqlx::PgPool;

use crate::{
    core::types::{MangaQuery, MangaSource},
    db::{inquiry::get_manga_paginated, model::MangaRow, update::update_manga_batch},
};

#[derive(Debug)]
pub enum DiffingResult {
    NoChange,
    Upcoming(MangaRow),
    Released(MangaRow),
}

pub async fn update_series(webhook_url: String, pool: &PgPool) {
    // retrieve series from db (paginated) based on the current day
    // for each series check for latest update
    let mut page_counter = 1;
    let mut all_series: Vec<MangaRow> = Vec::new();

    loop {
        println!("fetching series from db: page {}", page_counter);
        let series = get_manga_paginated(page_counter, 10, MangaQuery::default(), pool).await;

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
    let lim: Arc<DefaultKeyedRateLimiter<MangaSource>> = Arc::new(RateLimiter::keyed(
        Quota::per_second(NonZeroU32::new(2).unwrap()),
    ));
    let mut tasks = vec![];
    let mut task_output = vec![];

    for series in all_series {
        tasks.push(tokio::spawn(diff_update(series, lim.clone())));
    }

    for task in tasks {
        let task_result = task.await;

        match task_result {
            Ok(diff) => {
                task_output.push(diff);
            }
            Err(e) => {
                println!("Error : {}", e);
            }
        }
    }

    let rows: Vec<_> = task_output
        .iter()
        .filter_map(|d| match d {
            DiffingResult::NoChange => None,
            DiffingResult::Upcoming(manga_row) => Some(manga_row),
            DiffingResult::Released(manga_row) => Some(manga_row),
        })
        .collect();

    // update table
    if !rows.is_empty() {
        update_manga_batch(rows.into_iter(), pool)
            .await
            .expect("Error updating manga details");

        // broadcast diff change to webhook and update database
        broadcast_diff(&webhook_url, task_output).await;
    }

    println!("Update series job finished")
}

pub async fn broadcast_diff(webhook_url: &str, diffs: Vec<DiffingResult>) {
    let embeds = diffs.iter().filter_map(|d| match d {
        DiffingResult::NoChange => None,
        DiffingResult::Upcoming(manga) => Some(
            CreateEmbed::new()
                /*                 .url(&manga.latest_chapter_url) */
                .title(format!("[UPCOMING] {}", &manga.latest_chapter_title))
                .image(&manga.cover_url)
                .field(
                    "release_date",
                    format!(
                        "{}",
                        &manga
                            .latest_chapter_release_date
                            .format("%d-%m-%Y %H:%M:%S")
                    ),
                    false,
                )
                .field("series_name", &manga.title, false)
                .field("source", manga.source.to_string(), false)
                .field("author", &manga.author, false),
        ),
        DiffingResult::Released(manga) => Some(
            CreateEmbed::new()
                .url(&manga.latest_chapter_url)
                .title(format!("[RELEASED] {}", &manga.latest_chapter_title))
                .image(&manga.cover_url)
                .field("series_name", &manga.title, false)
                .field("source", manga.source.to_string(), false)
                .field("author", &manga.author, false),
        ),
    });

    let http = Http::new("");
    let webhook = Webhook::from_url(&http, webhook_url)
        .await
        .expect("Invalid webhook url");

    for embed in embeds {
        webhook
            .execute(&http, true, ExecuteWebhook::new().embed(embed))
            .await
            .expect("Error hitting webhook");

        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

pub async fn diff_update(
    data: MangaRow,
    limiter: Arc<DefaultKeyedRateLimiter<MangaSource>>,
) -> DiffingResult {
    limiter
        .until_key_ready_with_jitter(
            &data.source,
            Jitter::new(Duration::from_secs(1), Duration::from_secs(1)),
        )
        .await;

    let source = data.source.clone();

    // for each mangarow retrieve latest update
    let latest_update = data
        .source
        .fetch(&data.manga_id)
        .await
        .unwrap_or_else(|e| panic!("Fail to fetch {}: {:?}", source, e));

    // generate diffing result
    // no change -> chapter title and release status doesn't change
    // upcoming -> release status change from released to not released and chapter title change
    // released -> release status change from not released to released

    let update_manga_row = MangaRow::from_manga(data.manga_id, data.source, latest_update);

    if (data
        .latest_chapter_title
        .ne(&update_manga_row.latest_chapter_title)
        || !data.latest_chapter_released)
        && update_manga_row.latest_chapter_released
    {
        DiffingResult::Released(update_manga_row)
    } else if data
        .latest_chapter_title
        .ne(&update_manga_row.latest_chapter_title)
        && !update_manga_row.latest_chapter_released
    {
        DiffingResult::Upcoming(update_manga_row)
    } else {
        DiffingResult::NoChange
    }
}
