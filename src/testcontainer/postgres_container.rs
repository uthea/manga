use maybe_once::tokio::{Data, MaybeOnceAsync};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::Pool;
use std::sync::OnceLock;
use testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt};
use testcontainers_modules::postgres::Postgres;

use crate::testcontainer::postgres_container;

async fn init_postgres_container() -> ContainerAsync<Postgres> {
    Postgres::default()
        .with_tag("13-alpine")
        .start()
        .await
        .unwrap()
}

pub async fn get_postgres_container() -> Data<'static, ContainerAsync<Postgres>> {
    static POSTGRES_CONTAINER: OnceLock<MaybeOnceAsync<ContainerAsync<Postgres>>> = OnceLock::new();
    POSTGRES_CONTAINER
        .get_or_init(|| MaybeOnceAsync::new(|| Box::pin(init_postgres_container())))
        .data(false)
        .await
}

pub async fn get_test_db(
    db_name: &str,
) -> Result<
    (
        Pool<sqlx::Postgres>,
        Data<'static, ContainerAsync<testcontainers_modules::postgres::Postgres>>,
    ),
    sqlx::Error,
> {
    let container = postgres_container::get_postgres_container().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();
    let host_ip = container.get_host().await.unwrap().to_string();

    let options = PgConnectOptions::new()
        .username("postgres")
        .password("postgres")
        .host(&host_ip)
        .port(host_port)
        .ssl_mode(sqlx::postgres::PgSslMode::Disable);

    let pool = PgPoolOptions::new()
        .min_connections(1)
        .connect_with(options.clone())
        .await?;

    sqlx::query(format!(r#"create database "{db_name}""#).as_str())
        .execute(&pool)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    pool.set_connect_options(options.database(db_name));
    Ok((pool, container))
}
