use end2end::get_website_url;
use end2end::selenium_test_container;
use fantoccini::ClientBuilder;
use fantoccini::Locator;

#[ctor::ctor]
fn on_startup() {
    selenium_test_container::setup_selenium();
}

#[ctor::dtor]
fn on_shutdown() {
    selenium_test_container::shutdown_selenium();
}

#[tokio::test]
async fn access_website() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    let url = get_website_url();
    let selenium_port = selenium_test_container::get_selenium_node_port().await;
    let selenium_host = selenium_test_container::get_selenium_node_host().await;
    dbg!(&selenium_host);
    let c = ClientBuilder::native()
        .connect(format!("http://{}:{}", &selenium_host, selenium_port).as_str())
        .await?;

    // navigate to dashboard
    c.goto(format!("{}/dashboard", url).as_str()).await?;

    //check header
    let header = c.find(Locator::Css("body > div:nth-child(1) > main > div > div > div.thaw-scrollbar__container > div > div > div.thaw-layout-header > p")).await?;

    assert_eq!(header.text().await?, "Manga Tracker");

    Ok(())
}
