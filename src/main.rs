use axum::{
    extract::{Path, Request, State},
    response::IntoResponse,
    routing::get,
};
use leptos::{context::provide_context, logging::log};
use leptos_axum::handle_server_fns_with_context;
use manga_tracker::{app::shell, job::series::update_books, state::AppState};
use sqlx::migrate::Migrator;
use underway::{Job, To};

async fn load_db() -> Result<sqlx::PgPool, sqlx::Error> {
    use std::env;

    use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

    let username = env::var("DB_USERNAME").expect("DB_USERNAME is not set");
    let password = env::var("DB_PASSWORD").expect("DB_PASSWORD is not set");
    let host = env::var("DB_HOST").expect("DB_HOST is not set");
    let db_name = env::var("DB_NAME").expect("DB_NAME is not set");

    let options = PgConnectOptions::new()
        .host(&host)
        .username(&username)
        .password(&password)
        .database(&db_name);

    let pool = PgPoolOptions::new()
        .min_connections(5)
        .connect_with(options)
        .await?;

    let app_migrator = sqlx::migrate!("./migrations");

    let combined_migrations: Vec<_> = app_migrator
        .iter()
        .chain(underway::MIGRATOR.iter())
        .cloned()
        .collect();

    let combined_migrator = Migrator {
        migrations: combined_migrations.into(), // semver-exempt (!)
        ..Migrator::DEFAULT
    };

    combined_migrator.run(&pool).await?;

    Ok(pool)
}

async fn spawn_background_job(
    webhook_url: String,
    cron_expression: &str,
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let cloned_pool = pool.clone();

    let job = Job::builder()
        .step(move |_ctx, _input| {
            let url = webhook_url.clone();
            let cloned_pool = cloned_pool.clone();

            async move {
                update_books(url, &cloned_pool).await;
                To::done()
            }
        })
        .name("scheduled")
        .pool(pool.clone())
        .build()
        .await?;

    let cron = cron_expression.parse()?;
    job.schedule(&cron, &()).await?;

    tokio::spawn(async move {
        job.run().await.expect("Error running job");
    });

    Ok(())
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

#[tokio::main]
async fn main() {
    use std::env;

    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use manga_tracker::app::*;

    match env::var("APP_ENV") {
        Ok(e) if e == "prod" => (),
        _ => {
            dotenvy::dotenv().expect("can't load .env file");
        }
    };

    let db_pool = load_db().await.expect("Fail loading db connection");
    let webhook_url = env::var("WEBHOOK_URL").expect("WEBHOOK_URL is not set");
    let cron_expression = env::var("CRON_EXPR").expect("CRON_EXPR is not set");
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let app_state = AppState {
        leptos_options,
        pool: db_pool.clone(),
    };

    let app = Router::new()
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

    spawn_background_job(webhook_url, &cron_expression, db_pool.clone())
        .await
        .expect("Fail at spawning background job");

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
