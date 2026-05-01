pub mod server_info;
pub mod rcon;
pub mod logs;

use axum::{Router, routing::get};
use sqlx::PgPool;
use tokio::sync::broadcast;
use crate::models::server_log::LogEntry;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub log_broadcast: Option<Arc<broadcast::Sender<LogEntry>>>,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/servers", get(server_info::list))
        .route("/api/v1/servers/{id}", get(server_info::get_one).put(server_info::update))
        .route("/api/v1/servers/{id}/rcon", axum::routing::post(rcon::execute))
        .route("/api/v1/servers/{id}/rcon-logs", get(rcon::list_logs))
        .route("/api/v1/servers/{id}/logs", get(logs::list))
        .route("/api/v1/servers/{id}/logs/stream", get(logs::stream))
        .with_state(state)
}
