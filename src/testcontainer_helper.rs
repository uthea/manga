// taken from https://github.com/lloydmeta/miniaturs/blob/d244760f5039a15450f5d4566ffe52d19d427771/server/src/test_utils/mod.rs

use std::thread;
use testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt};
use tokio::sync::mpsc;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex, OnceCell,
};

use testcontainers_modules::postgres::Postgres;

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

static POSTGRES_NODE: OnceCell<Mutex<Option<ContainerAsync<Postgres>>>> = OnceCell::const_new();

async fn postgres_node() -> &'static Mutex<Option<ContainerAsync<Postgres>>> {
    POSTGRES_NODE
        .get_or_init(|| async {
            let container = Postgres::default()
                .with_tag("13-alpine")
                .start()
                .await
                .unwrap();

            Mutex::new(Some(container))
        })
        .await
}

pub async fn get_postgres_node_port() -> u16 {
    postgres_node()
        .await
        .lock()
        .await
        .as_ref()
        .unwrap()
        .get_host_port_ipv4(5432)
        .await
        .unwrap()
}

pub async fn get_postgres_node_host() -> String {
    postgres_node()
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

async fn drop_postgres_node() {
    postgres_node()
        .await
        .lock()
        .await
        .take()
        .unwrap()
        .rm()
        .await
        .unwrap();
}

static POSTGRES_CHANNEL: std::sync::OnceLock<Channel<ContainerCommands>> =
    std::sync::OnceLock::new();
fn postgres_channel() -> &'static Channel<ContainerCommands> {
    POSTGRES_CHANNEL.get_or_init(channel)
}

static POSTGRES_SHUT_DOWN_NOTIFIER_CHANNEL: std::sync::OnceLock<Channel<()>> =
    std::sync::OnceLock::new();
fn postgres_shut_down_notifier_channel() -> &'static Channel<()> {
    POSTGRES_SHUT_DOWN_NOTIFIER_CHANNEL.get_or_init(channel)
}

static TOKIO_RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
fn tokio_runtime() -> &'static tokio::runtime::Runtime {
    TOKIO_RUNTIME.get_or_init(|| tokio::runtime::Runtime::new().unwrap())
}

async fn start_postgres() {
    let mut rx = postgres_channel().rx.lock().await;
    while let Some(command) = rx.recv().await {
        match command {
            ContainerCommands::Stop => {
                drop_postgres_node().await;
                rx.close();
            }
        }
    }
}

pub fn shutdown_postgres() {
    postgres_channel()
        .tx
        .blocking_send(ContainerCommands::Stop)
        .unwrap();
    postgres_shut_down_notifier_channel()
        .rx
        .blocking_lock()
        .blocking_recv()
        .unwrap();
}

pub fn setup_postgres() {
    thread::spawn(|| {
        tokio_runtime().block_on(start_postgres());
        // This needs to be here otherwise the container did not call the drop function before the application stops
        postgres_shut_down_notifier_channel()
            .tx
            .blocking_send(())
            .unwrap();
    });
}
