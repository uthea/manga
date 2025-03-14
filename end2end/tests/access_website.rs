use end2end::selenium_test_container;
use thirtyfour::{By, DesiredCapabilities, WebDriver};

#[ctor::ctor]
fn on_startup() {
    selenium_test_container::setup_selenium();
}

#[ctor::dtor]
fn on_shutdown() {
    selenium_test_container::shutdown_selenium();
}

// #[cfg(target_arch = "aarch64")]
// const WEBSITE_URL: &str = "http://host.docker.internal:3000/dashboard";
//
// #[cfg(not(target_arch = "aarch64"))]
// const WEBSITE_URL: &'static str = "http://172.17.0.1:3000/dashboard";

#[tokio::test]
async fn access_website() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;

    let selenium_port = selenium_test_container::get_selenium_node_port().await;
    let selenium_host = selenium_test_container::get_selenium_node_host().await;
    let caps = DesiredCapabilities::chrome();
    dbg!(&selenium_host);
    let driver =
        WebDriver::new(format!("http://{}:{}", &selenium_host, selenium_port), caps).await?;

    // navigate to dashboard
    driver
        .goto(format!("http://{}:{}", &selenium_host, selenium_port))
        .await?;

    //check header
    let header = driver.find(By::Css("body > div:nth-child(1) > main > div > div > div.thaw-scrollbar__container > div > div > div.thaw-layout-header > p")).await?;

    assert_eq!(header.text().await?, "Manga Tracker");

    Ok(())
}
