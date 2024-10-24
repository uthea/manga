-- Add migration script here
create table series (
    source MangaSource not null, 
    manga_id text not null,
    title text not null,
    cover_url text not null,
    author text not null,
    latest_chapter_title text not null,
    latest_chapter_url text not null,
    latest_chapter_release_date timestamp not null,
    latest_chapter_publish_day Weekday not null,
    latest_chapter_released boolean not null,
    last_update timestamp not null,
    PRIMARY KEY(source, manga_id)
)
