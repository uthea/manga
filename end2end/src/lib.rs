pub mod selenium_test_container;

pub(crate) fn tokio_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Runtime::new().unwrap()
}

pub fn get_website_url() -> String {
    #[cfg(target_arch = "aarch64")]
    let website_url: String = "http://host.docker.internal:3000".into();

    #[cfg(not(target_arch = "aarch64"))]
    let website_url: String = "http://172.17.0.1:3000".into();

    website_url
}

pub async fn get_selenium_driver_url() -> String {
    let selenium_port = selenium_test_container::get_selenium_node_port().await;
    let selenium_host = selenium_test_container::get_selenium_node_host().await;

    format!("http://{}:{}", &selenium_host, selenium_port)
}
