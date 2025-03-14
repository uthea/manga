use axum::{
    extract::{Path, Request, State},
    response::IntoResponse,
    routing::get,
};
use leptos::{context::provide_context, logging::log};
use leptos_axum::handle_server_fns_with_context;
use manga_tracker::{app::shell, job::series::update_series, state::AppState};

async fn load_db() -> Result<sqlx::PgPool, sqlx::Error> {
    use std::env;

    use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

    let options = {
        let mut username = "postgres".to_string();
        let mut password = "postgres".to_string();
        let mut host = "localhost".to_string();
        let mut db_name = "postgres".to_string();

        if env::var("E2E_TEST").is_err() {
            username = env::var("DB_USERNAME").expect("DB_USERNAME is not set");
            password = env::var("DB_PASSWORD").expect("DB_PASSWORD is not set");
            host = env::var("DB_HOST").expect("DB_HOST is not set");
            db_name = env::var("DB_NAME").expect("DB_NAME is not set");
        }

        if env::var("GA").is_ok() {
            host = "172.17.0.1".to_string();
        }

        if env::var("E2E_TEST").is_ok() {
            PgConnectOptions::new()
                .username(&username)
                .password(&password)
                .database(&db_name)
                .host(&host)
                .port(1234)
                .ssl_mode(sqlx::postgres::PgSslMode::Disable)
        } else {
            PgConnectOptions::new()
                .host(&host)
                .username(&username)
                .password(&password)
                .database(&db_name)
        }
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

    match env::var("APP_ENV") {
        Ok(e) if e == "prod" => (),
        Ok(e) if e == "test" => (),
        _ => {
            dotenvy::dotenv().expect("can't load .env file");
        }
    };

    let db_pool = load_db().await.expect("Fail loading db connection");
    let webhook_url = env::var("WEBHOOK_URL").expect("WEBHOOK_URL is not set");
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;

    if let Some(arg) = env::args().nth(1) {
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
        .await
        .unwrap();
}
