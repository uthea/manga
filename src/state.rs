use axum::extract::FromRef;
use leptos::config::LeptosOptions;
use sqlx::PgPool;

#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub pool: PgPool,
}
