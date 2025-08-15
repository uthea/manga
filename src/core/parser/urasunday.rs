use crate::core::{fetch::FetchError, types::Manga};
use chrono::{Datelike, Days, Local, NaiveDate, NaiveTime, TimeZone};
use chrono_tz::Japan;
use fantoccini::ClientBuilder;
use reqwest::Client;
use scraper::{Html, Selector};

pub fn parse_urasunday_from_html(html: String, manga_id: &str) -> Result<Manga, FetchError> {
    let document = Html::parse_document(&html);

    let title_selector = Selector::parse(r#"#aboutTitle > div > div > div > h2"#).unwrap();
    let author_selector = Selector::parse(r#"#aboutTitle > div > div > div > div > p"#).unwrap();
    let chapter_selector =
        Selector::parse(r#"div[class="flex gap-3 switch:items-center items-center"]"#).unwrap();
    let chapter_title_selector = Selector::parse(r#"div[class="grow"] > p"#).unwrap();
    let chapter_release_date_selector =
        Selector::parse(r#"p[class="text-xs text-gray-700 switch:text-base"]"#).unwrap();
    let chapter_img_selector = Selector::parse(r#"img[class*="object-cover"]"#).unwrap();
    let chapter_not_released_selector = Selector::parse(r#"div[class="flex aspect-square h-6 items-center justify-center rounded-[3px] text-[9px] text-white switch:h-8 switch:rounded switch:text-xs bg-gradient-orange"] > span"#).unwrap();

    let title = document
        .select(&title_selector)
        .next()
        .ok_or(FetchError::PageNotFound(Some(
            "series title not found".into(),
        )))?
        .inner_html();
    let author = document
        .select(&author_selector)
        .next()
        .ok_or(FetchError::PageNotFound(Some("author not found".into())))?
        .inner_html();

    let mut latest_chapter_iter = document.select(&chapter_selector);
    latest_chapter_iter.next();

    if let Some(lastest_chapter_fragment) = latest_chapter_iter.next() {
        let chapter_title = lastest_chapter_fragment
            .select(&chapter_title_selector)
            .map(|e| e.inner_html())
            .reduce(|acc, s| format!("{acc} {s}"))
            .ok_or(FetchError::ChapterNotFound(Some(
                "chapter title not found".into(),
            )))?;

        let chapter_release_date = {
            let mut date = Local::now().checked_add_days(Days::new(1)).unwrap();

            if lastest_chapter_fragment
                .select(&chapter_not_released_selector)
                .next()
                .is_none()
            {
                let raw = lastest_chapter_fragment
                    .select(&chapter_release_date_selector)
                    .next()
                    .ok_or(FetchError::ChapterNotFound(Some(
                        "release date not found".into(),
                    )))?
                    .inner_html();

                let naive_date = NaiveDate::parse_from_str(&raw, "%Y/%m/%d").map_err(|e| {
                    FetchError::ChapterNotFound(Some(format!(
                        "Error parsing date {} : {}",
                        &raw, e
                    )))
                })?;

                date = Local
                    .from_local_datetime(&naive_date.and_time(NaiveTime::default()))
                    .unwrap();
            }

            date
        };

        let chapter_img = lastest_chapter_fragment
            .select(&chapter_img_selector)
            .next()
            .ok_or(FetchError::ChapterNotFound(Some("cover not found".into())))?
            .attr("src")
            .ok_or(FetchError::ChapterNotFound(Some(
                "cover not found in src attribute".into(),
            )))?;

        let chapter_id = "TODO";

        return Ok(Manga {
            title: title.trim().into(),
            cover_url: chapter_img.into(),
            author: author.trim().into(),
            latest_chapter_title: chapter_title,
            latest_chapter_url: format!(
                "https://manga-one.com/manga/{manga_id}/chapter/{chapter_id}"
            ),
            latest_chapter_release_date: chapter_release_date.fixed_offset(),
            latest_chapter_publish_day: chapter_release_date.with_timezone(&Japan).weekday(),
        });
    }

    Err(FetchError::ChapterNotFound(None))
}

pub fn parse_chapter_id_from_url(url: &str) -> Result<&str, FetchError> {
    let remove_prefix = url
        .split("chapter/")
        .last()
        .ok_or(FetchError::ChapterNotFound(Some(
            "prefix split not foud".into(),
        )))?;

    remove_prefix
        .split(".webp")
        .next()
        .ok_or(FetchError::ChapterNotFound(Some(
            "suffix split not foud".into(),
        )))
}

pub async fn fetch_urasunday(webdriver_url: &str, manga_id: &str) -> Result<Manga, FetchError> {
    let url = format!("https://urasunday.com/title/{manga_id}/chapter/1234");
    let wv_client = ClientBuilder::native()
        .connect(webdriver_url)
        .await
        .map_err(FetchError::WebDriverSessionError)?;

    wv_client
        .goto(&url)
        .await
        .map_err(FetchError::WebDriverCmdError)?;
    let html = wv_client
        .source()
        .await
        .map_err(FetchError::WebDriverCmdError)?;

    parse_urasunday_from_html(html, manga_id)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::testcontainer::selenium_container;

    use super::*;

    // Setup hooks registration
    #[ctor::ctor]
    fn on_startup() {
        selenium_container::setup_selenium();
    }

    // Shutdown hook registration
    #[ctor::dtor]
    fn on_shutdown() {
        selenium_container::shutdown_selenium();
    }

    #[test]
    fn test_parse_urasunday_source() {
        let paths = fs::read_dir("src/test_data/urasunday").unwrap();

        for path in paths {
            dbg!(&path);
            let html = fs::read_to_string(path.unwrap().path()).unwrap();
            let _ = parse_urasunday_from_html(html, "").unwrap();
        }
    }

    #[test]
    fn urasunday_parse_chapter_id_from_url() {
        let url = "https://app.manga-one.com/secure/1754030157/webp/chapter/300898.webp?hash=UN0kclezTZQepugoVuOyHw&expires=1841184000";
        let expected = "300898";

        let result = parse_chapter_id_from_url(url).unwrap();

        assert_eq!(expected, result);
    }

    #[ignore]
    #[tokio::test]
    async fn fetch_urasunday_test() {
        let url = "https://urasunday.com/title/939/chapter/1234";
        let selenium_port = selenium_container::get_selenium_node_port().await;
        let selenium_host = selenium_container::get_selenium_node_host().await;

        let wv_client = ClientBuilder::native()
            .connect(format!("http://{}:{}", &selenium_host, selenium_port).as_ref())
            .await
            .unwrap();

        wv_client.goto(url).await.unwrap();
        let html = wv_client.source().await.unwrap();

        parse_urasunday_from_html(html, "939").unwrap();
    }
}
