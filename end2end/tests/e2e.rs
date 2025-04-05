use end2end::get_selenium_driver_url;
use end2end::get_website_url;
use end2end::selenium_test_container;
use fantoccini::ClientBuilder;
use fantoccini::Locator;

#[ctor::ctor]
fn on_startup() {
    selenium_test_container::setup_selenium();
    color_eyre::install().unwrap();
}

#[ctor::dtor]
fn on_shutdown() {
    selenium_test_container::shutdown_selenium();
}

#[tokio::test]
async fn access_website() -> color_eyre::eyre::Result<()> {
    let url = get_website_url();
    let driver_url = get_selenium_driver_url().await;
    let c = ClientBuilder::native().connect(driver_url.as_str()).await?;

    // navigate to dashboard
    c.goto(format!("{}/dashboard", url).as_str()).await?;

    //check header
    let header = c.find(Locator::Css("body > div:nth-child(1) > main > div > div > div.thaw-scrollbar__container > div > div > div.thaw-layout-header > p")).await?;

    assert_eq!(header.text().await?, "Manga Tracker");

    c.close().await?;

    Ok(())
}

#[tokio::test]
async fn delete_existing_source() -> color_eyre::eyre::Result<()> {
    let url = get_website_url();
    let driver_url = get_selenium_driver_url().await;
    let c = ClientBuilder::native().connect(driver_url.as_str()).await?;

    let id_1 = "Young Champion-856798df666ae";
    let id_2 = "YanMaga-彼女の友達";

    // navigate to dashboard
    c.goto(format!("{}/dashboard", url).as_str()).await?;

    // select multiple source for delete
    let cbox_1 = c
        .find(Locator::Id(format!("del-{}", id_1).as_str()))
        .await?;
    let cbox_2 = c
        .find(Locator::Id(format!("del-{}", id_2).as_str()))
        .await?;

    cbox_1.click().await?;
    cbox_2.click().await?;

    // pop up delete dialog
    let delete_btn = c.find(Locator::Id("trigger-delete-dialog-btn")).await?;
    delete_btn.click().await?;

    // confirm delete
    let dialog_delete_btn = c.find(Locator::Id("delete-dialog-delete-btn")).await?;
    dialog_delete_btn.click().await?;

    // row should not exist
    while c
        .find(Locator::Id(format!("row-{}", id_1).as_str()))
        .await
        .is_ok()
    {}

    while c
        .find(Locator::Id(format!("row-{}", id_2).as_str()))
        .await
        .is_ok()
    {}

    c.close().await?;

    Ok(())
}

#[tokio::test]
async fn add_new_source() -> color_eyre::eyre::Result<()> {
    let url = get_website_url();
    let driver_url = get_selenium_driver_url().await;
    let c = ClientBuilder::native().connect(driver_url.as_str()).await?;

    let manga_id_selector = "#add-dialog-manga-id > input";
    let manga_source_selector = "#add-dialog-source > input";
    let add_btn_id = "trigger-add-dialog-btn";
    let submit_btn_id = "add-dialog-add-btn";

    // navigate to dashboard
    c.goto(format!("{}/dashboard", url).as_str()).await?;

    // pop up add dialog
    let add_btn = c.find(Locator::Id(add_btn_id)).await?;
    add_btn.click().await?;
    dbg!("pop up add dialog");

    // fill manga id and source
    let manga_id = c.find(Locator::Css(manga_id_selector)).await?;
    let manga_source = c.find(Locator::Css(manga_source_selector)).await?;

    manga_id.send_keys("10834108156641784251").await?;
    manga_source.send_keys("Shounen\n").await?;
    dbg!("fill source form");

    // submit
    c.find(Locator::Id(submit_btn_id)).await?.click().await?;
    dbg!("submit");

    // check row id
    c.wait()
        .for_element(Locator::Id("row-Shounen Jump Plus-10834108156641784251"))
        .await?;

    c.close().await?;

    Ok(())
}

#[tokio::test]
async fn add_duplicate_source_toast_error() -> color_eyre::eyre::Result<()> {
    let url = get_website_url();
    let driver_url = get_selenium_driver_url().await;
    let c = ClientBuilder::native().connect(driver_url.as_str()).await?;

    let manga_id_selector = "#add-dialog-manga-id > input";
    let manga_source_selector = "#add-dialog-source > input";
    let add_btn_id = "trigger-add-dialog-btn";
    let submit_btn_id = "add-dialog-add-btn";

    // navigate to dashboard
    c.goto(format!("{}/dashboard", url).as_str()).await?;

    // pop up add dialog
    let add_btn = c.find(Locator::Id(add_btn_id)).await?;
    add_btn.click().await?;
    dbg!("pop up add dialog");

    // fill manga id and source
    let manga_id = c.find(Locator::Css(manga_id_selector)).await?;
    let manga_source = c.find(Locator::Css(manga_source_selector)).await?;

    manga_id
        .send_keys("股間無双嫌われ勇者は魔族に愛される")
        .await?;
    manga_source.send_keys("YanMaga\n").await?;
    dbg!("fill source form");

    // submit
    c.find(Locator::Id(submit_btn_id)).await?.click().await?;
    dbg!("submit");

    // check for error toast
    c.wait().for_element(Locator::Id("toast-add-error")).await?;

    c.close().await?;

    Ok(())
}

#[tokio::test]
async fn filter_by_source() -> color_eyre::eyre::Result<()> {
    let url = get_website_url();
    let driver_url = get_selenium_driver_url().await;
    let c = ClientBuilder::native().connect(driver_url.as_str()).await?;

    let source_filter_trigger_id = "source-filter-trigger";
    let manga_source_filter_selector = "#source-filter-select > input";

    // navigate to dashboard
    c.goto(format!("{}/dashboard", url).as_str()).await?;

    // trigger menu dropwon
    c.find(Locator::Id(source_filter_trigger_id))
        .await?
        .click()
        .await?;

    // filter manga source
    let manga_source = c.find(Locator::Css(manga_source_filter_selector)).await?;
    manga_source.send_keys("Ichijin\n").await?;
    dbg!("fill source");

    // tr element must be one
    loop {
        let rows_len = c.find_all(Locator::Css("tbody > tr")).await?.len();
        if rows_len == 1 {
            break;
        }
    }

    // check if filtered row is correct
    c.wait()
        .for_element(Locator::Id("row-Ichijin Plus-2410174332975"))
        .await?;

    c.close().await?;

    Ok(())
}

#[tokio::test]
async fn filter_by_title() -> color_eyre::eyre::Result<()> {
    let url = get_website_url();
    let driver_url = get_selenium_driver_url().await;
    let c = ClientBuilder::native().connect(driver_url.as_str()).await?;

    let filter_trigger_id = "title-filter-trigger";
    let filter_input_selector = "#title-filter-input > input";

    // navigate to dashboard
    c.goto(format!("{}/dashboard", url).as_str()).await?;

    // trigger menu dropdown
    c.find(Locator::Id(filter_trigger_id))
        .await?
        .click()
        .await?;

    // filter
    let filter_input = c.find(Locator::Css(filter_input_selector)).await?;
    filter_input.send_keys("おかえり、パパ\n").await?;

    // tr element must be one
    loop {
        let rows_len = c.find_all(Locator::Css("tbody > tr")).await?.len();
        if rows_len == 1 {
            break;
        }
    }

    // check if filtered row is correct
    c.wait()
        .for_element(Locator::Id("row-Champion Cross-c887451b8309e"))
        .await?;

    c.close().await?;

    Ok(())
}

#[tokio::test]
async fn filter_by_author() -> color_eyre::eyre::Result<()> {
    let url = get_website_url();
    let driver_url = get_selenium_driver_url().await;
    let c = ClientBuilder::native().connect(driver_url.as_str()).await?;

    let filter_trigger_id = "author-filter-trigger";
    let filter_input_selector = "#author-filter-input > input";

    // navigate to dashboard
    c.goto(format!("{}/dashboard", url).as_str()).await?;

    // trigger menu dropdown
    c.find(Locator::Id(filter_trigger_id))
        .await?
        .click()
        .await?;

    // filter
    let filter_input = c.find(Locator::Css(filter_input_selector)).await?;
    filter_input.send_keys("あおのなち\n").await?;

    // tr element must be one
    loop {
        let rows_len = c.find_all(Locator::Css("tbody > tr")).await?.len();
        if rows_len == 1 {
            break;
        }
    }

    // check if filtered row is correct
    c.wait()
        .for_element(Locator::Id("row-Ichijin Plus-2410174332975"))
        .await?;

    c.close().await?;

    Ok(())
}

#[tokio::test]
async fn filter_by_chapter_name() -> color_eyre::eyre::Result<()> {
    let url = get_website_url();
    let driver_url = get_selenium_driver_url().await;
    let c = ClientBuilder::native().connect(driver_url.as_str()).await?;

    let filter_trigger_id = "chapter-filter-trigger";
    let filter_input_selector = "#chapter-filter-input > input";

    // navigate to dashboard
    c.goto(format!("{}/dashboard", url).as_str()).await?;

    // trigger menu dropdown
    c.find(Locator::Id(filter_trigger_id))
        .await?
        .click()
        .await?;

    // filter
    let filter_input = c.find(Locator::Css(filter_input_selector)).await?;
    filter_input.send_keys("次回公開予定\n").await?;

    // tr element must be one
    loop {
        let rows_len = c.find_all(Locator::Css("tbody > tr")).await?.len();
        if rows_len == 1 {
            break;
        }
    }

    // check if filtered row is correct
    c.wait()
        .for_element(Locator::Id(
            "row-YanMaga-股間無双嫌われ勇者は魔族に愛される",
        ))
        .await?;

    c.close().await?;

    Ok(())
}
