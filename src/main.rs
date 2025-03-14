use axum::{
    extract::{Path, Request, State},
    response::IntoResponse,
    routing::get,
};
use leptos::{context::provide_context, logging::log};
use leptos_axum::handle_server_fns_with_context;
use manga_tracker::{app::shell, job::series::update_series, state::AppState};
use testcontainers::{runners::AsyncRunner, ImageExt};
use testcontainers_modules::postgres::Postgres;
use tokio::signal;

async fn load_db() -> Result<sqlx::PgPool, sqlx::Error> {
    use std::env;

    use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

    let options = {
        let username = env::var("DB_USERNAME").expect("DB_USERNAME is not set");
        let password = env::var("DB_PASSWORD").expect("DB_PASSWORD is not set");
        let host = env::var("DB_HOST").expect("DB_HOST is not set");
        let db_name = env::var("DB_NAME").expect("DB_NAME is not set");

        PgConnectOptions::new()
            .host(&host)
            .username(&username)
            .password(&password)
            .database(&db_name)
    };

    let pool = PgPoolOptions::new()
        .min_connections(5)
        .connect_with(options)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

async fn load_db_test(host: String, port: u16) -> Result<sqlx::PgPool, sqlx::Error> {
    use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

    let options = {
        let username = "postgres".to_string();
        let password = "postgres".to_string();
        let db_name = "postgres".to_string();

        PgConnectOptions::new()
            .host(&host)
            .username(&username)
            .password(&password)
            .database(&db_name)
            .port(port)
    };

    let pool = PgPoolOptions::new()
        .min_connections(5)
        .connect_with(options)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

async fn server_fn_handler(
    State(app_state): State<AppState>,
    _path: Path<String>,
    request: Request<axum::body::Body>,
) -> impl IntoResponse {
    handle_server_fns_with_context(
        move || {
            provide_context(app_state.clone());
            provide_context(app_state.pool.clone());
        },
        request,
    )
    .await
}

pub async fn leptos_routes_handler(
    State(app_state): State<AppState>,
    request: Request<axum::body::Body>,
) -> axum::response::Response {
    let options = app_state.leptos_options.clone();
    let handler = leptos_axum::render_app_async_with_context(
        move || {
            provide_context(app_state.clone());
            provide_context(app_state.pool.clone());
        },
        move || shell(options.clone()),
    );

    handler(request).await.into_response()
}

async fn health() -> &'static str {
    "."
}

#[tokio::main]
async fn main() {
    use std::env;

    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use manga_tracker::app::*;

    let mut e2e_flag = false;

    match env::var("APP_ENV") {
        Ok(e) if e == "prod" => (),
        Ok(e) if e == "e2e" => e2e_flag = true,
        _ => {
            dotenvy::dotenv().expect("can't load .env file");
        }
    };

    let postgres_container = {
        if e2e_flag {
            Some(
                Postgres::default()
                    .with_tag("13-alpine")
                    .start()
                    .await
                    .unwrap(),
            )
        } else {
            None
        }
    };

    let db_pool = {
        if let Some(container) = postgres_container.as_ref() {
            let db_host = container.get_host().await.unwrap().to_string();
            let db_port = container.get_host_port_ipv4(5432).await.unwrap();
            load_db_test(db_host, db_port)
                .await
                .expect("Fail loading db connection")
        } else {
            load_db().await.expect("Fail loading db connection")
        }
    };
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;

    if let Some(arg) = env::args().nth(1) {
        let webhook_url = env::var("WEBHOOK_URL").expect("WEBHOOK_URL is not set");
        if arg == "update" {
            println!("start updating series");
            update_series(webhook_url, &db_pool).await;
            return;
        }
    }

    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let app_state = AppState {
        leptos_options,
        pool: db_pool.clone(),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route(
            "/api/*fn_name",
            get(server_fn_handler).post(server_fn_handler),
        )
        .leptos_routes_with_handler(routes, get(leptos_routes_handler))
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .with_state(app_state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => (),
        _ = terminate => (),
    }
}
