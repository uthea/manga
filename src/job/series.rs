use std::{num::NonZeroU32, sync::Arc, time::Duration};

use chrono::TimeZone;
use chrono_tz::Japan;
use governor::{DefaultKeyedRateLimiter, Jitter, Quota, RateLimiter};
use serenity::all::{CreateEmbed, ExecuteWebhook, Http, Webhook};
use sqlx::PgPool;

use crate::{
    core::{fetch::fetch_manga, manga::Manga, types::MangaSource},
    db::{
        inquiry::{get_manga_paginated, MangaQuery},
        model::MangaRow,
    },
};

#[derive(Debug)]
pub enum DiffingResult {
    NoChange,
    Upcoming(MangaSource, Manga),
    Released(MangaSource, Manga),
}

pub async fn update_books(webhook_url: String, pool: &PgPool) {
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

    // broadcast diff change to webhook and update database
    broadcast_diff(&webhook_url, task_output).await;

    // TODO: create query for db update
}

pub async fn broadcast_diff(webhook_url: &str, diffs: Vec<DiffingResult>) {
    let embeds = diffs.iter().filter_map(|d| match d {
        DiffingResult::NoChange => None,
        DiffingResult::Upcoming(source, manga) => Some(
            CreateEmbed::new()
                /*                 .url(&manga.latest_chapter_url) */
                .title(format!("[UPCOMING] {}", &manga.latest_chapter_title))
                .image(&manga.cover_url)
                .field("series_name", &manga.title, false)
                .field("source", source.to_string(), false)
                .field("author", &manga.author, false),
        ),
        DiffingResult::Released(source, manga) => Some(
            CreateEmbed::new()
                .url(&manga.latest_chapter_url)
                .title(format!("[RELEASED] {}", &manga.latest_chapter_title))
                .image(&manga.cover_url)
                .field("series_name", &manga.title, false)
                .field("source", source.to_string(), false)
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
        DiffingResult::Released(data.source, latest_update)
    } else if data
        .latest_chapter_title
        .ne(&latest_update.latest_chapter_title)
        && !released
    {
        DiffingResult::Upcoming(data.source, latest_update)
    } else {
        DiffingResult::NoChange
    }
}
