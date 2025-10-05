use maybe_once::tokio::{Data, MaybeOnceAsync};
use testcontainers::ImageExt;
use testcontainers::{core::WaitFor, Image};

use std::sync::OnceLock;
use testcontainers::{runners::AsyncRunner, ContainerAsync};

const NAME: &str = "selenium/standalone-chromium";
const TAG: &str = "138.0";

#[derive(Debug, Clone, Default)]
pub struct Selenium {}

impl Image for Selenium {
    fn name(&self) -> &str {
        NAME
    }

    fn tag(&self) -> &str {
        TAG
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![
            //WaitFor::message_on_stderr("Started Selenium Standalone"),
            WaitFor::message_on_stdout("Started Selenium Standalone"),
        ]
    }
}

async fn init_selenium_container() -> ContainerAsync<Selenium> {
    Selenium::default()
        .with_cmd(["/opt/bin/entry_point.sh", r#"--shm-size="2g""#])
        .start()
        .await
        .unwrap()
}

pub async fn get_selenium_container() -> Data<'static, ContainerAsync<Selenium>> {
    static SELENIUM_CONTAINER: OnceLock<MaybeOnceAsync<ContainerAsync<Selenium>>> = OnceLock::new();
    SELENIUM_CONTAINER
        .get_or_init(|| MaybeOnceAsync::new(|| Box::pin(init_selenium_container())))
        .data(false)
        .await
}

pub async fn get_selenium_info() -> (String, Data<'static, ContainerAsync<Selenium>>) {
    let selenium_ctr = get_selenium_container().await;
    let selenium_port = selenium_ctr.get_host_port_ipv4(4444).await.unwrap();
    let selenium_host = selenium_ctr.get_host().await.unwrap().to_string();

    (
        format!("http://{}:{}", &selenium_host, selenium_port),
        selenium_ctr,
    )
}

