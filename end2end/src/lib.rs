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
