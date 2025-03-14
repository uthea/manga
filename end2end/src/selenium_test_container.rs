use testcontainers::ImageExt;
use testcontainers::{core::WaitFor, Image};

use std::thread;
use testcontainers::{runners::AsyncRunner, ContainerAsync};
use tokio::sync::mpsc;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex, OnceCell,
};

use crate::tokio_runtime;

#[cfg(target_arch = "aarch64")]
const NAME: &str = "seleniarm/standalone-chromium";

#[cfg(not(target_arch = "aarch64"))]
const NAME: &str = "selenium/standalone-chromium";

#[cfg(target_arch = "aarch64")]
const TAG: &str = "124.0";

#[cfg(not(target_arch = "aarch64"))]
const TAG: &str = "133.0";

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

enum ContainerCommands {
    Stop,
}

struct Channel<T> {
    tx: Sender<T>,
    rx: Mutex<Receiver<T>>,
}

fn channel<T>() -> Channel<T> {
    let (tx, rx) = mpsc::channel(32);
    Channel {
        tx,
        rx: Mutex::new(rx),
    }
}

static SELENIUM_NODE: OnceCell<Mutex<Option<ContainerAsync<Selenium>>>> = OnceCell::const_new();

async fn selenium_node() -> &'static Mutex<Option<ContainerAsync<Selenium>>> {
    SELENIUM_NODE
        .get_or_init(|| async {
            let container = Selenium::default()
                .with_cmd([
                    "/opt/bin/entry_point.sh",
                    "--net=host",
                    r#"--shm-size="2g""#,
                ])
                .start()
                .await
                .unwrap();

            Mutex::new(Some(container))
        })
        .await
}

pub async fn get_selenium_node_port() -> u16 {
    selenium_node()
        .await
        .lock()
        .await
        .as_ref()
        .unwrap()
        .get_host_port_ipv4(4444)
        .await
        .unwrap()
}

pub async fn get_selenium_node_host() -> String {
    selenium_node()
        .await
        .lock()
        .await
        .as_ref()
        .unwrap()
        .get_host()
        .await
        .unwrap()
        .to_string()
}

async fn drop_selenium_node() {
    selenium_node()
        .await
        .lock()
        .await
        .take()
        .unwrap()
        .rm()
        .await
        .unwrap();
}

static SELENIUM_CHANNEL: std::sync::OnceLock<Channel<ContainerCommands>> =
    std::sync::OnceLock::new();
fn selenium_channel() -> &'static Channel<ContainerCommands> {
    SELENIUM_CHANNEL.get_or_init(channel)
}

static SELENIUM_SHUT_DOWN_NOTIFIER_CHANNEL: std::sync::OnceLock<Channel<()>> =
    std::sync::OnceLock::new();
fn selenium_shut_down_notifier_channel() -> &'static Channel<()> {
    SELENIUM_SHUT_DOWN_NOTIFIER_CHANNEL.get_or_init(channel)
}

async fn start_selenium() {
    let mut rx = selenium_channel().rx.lock().await;
    while let Some(command) = rx.recv().await {
        match command {
            ContainerCommands::Stop => {
                drop_selenium_node().await;
                rx.close();
            }
        }
    }
}

pub fn shutdown_selenium() {
    selenium_channel()
        .tx
        .blocking_send(ContainerCommands::Stop)
        .unwrap();
    selenium_shut_down_notifier_channel()
        .rx
        .blocking_lock()
        .blocking_recv()
        .unwrap();
}

pub fn setup_selenium() {
    thread::spawn(|| {
        tokio_runtime().block_on(start_selenium());
        // This needs to be here otherwise the container did not call the drop function before the application stops
        selenium_shut_down_notifier_channel()
            .tx
            .blocking_send(())
            .unwrap();
    });
}
