use end2end::{postgres_test_container, selenium_test_container};
use thirtyfour::{By, DesiredCapabilities, WebDriver};

#[ctor::ctor]
fn on_startup() {
    postgres_test_container::setup_postgres();
    selenium_test_container::setup_selenium();
}

#[ctor::dtor]
fn on_shutdown() {
    postgres_test_container::shutdown_postgres();
    selenium_test_container::shutdown_selenium();
}

#[cfg(target_arch = "aarch64")]
const WEBSITE_URL: &str = "http://host.docker.internal:3000";

#[cfg(not(target_arch = "aarch64"))]
const WEBSITE_URL: &'static str = "http://localhost:3000";

#[tokio::test]
async fn access_website() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;

    let selenium_port = selenium_test_container::get_selenium_node_port().await;
    let _postgres_port = postgres_test_container::get_postgres_node_port().await;
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new(format!("http://localhost:{}", selenium_port), caps).await?;

    // navigate to dashboard
    driver.goto(WEBSITE_URL).await?;

    //check header
    let header = driver.find(By::Css("body > div:nth-child(1) > main > div > div > div.thaw-scrollbar__container > div > div > div.thaw-layout-header > p")).await?;

    assert_eq!(header.text().await?, "Manga Tracker");

    Ok(())
}
