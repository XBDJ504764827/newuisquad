pub mod server_info;
pub mod rcon;
pub mod logs;
pub mod files;
pub mod agent_ws;
pub mod servers;

use axum::{Router, routing::{get, post}};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;
use crate::models::server_log::LogEntry;
use crate::api::agent_ws::AgentPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub log_broadcast: Option<Arc<broadcast::Sender<LogEntry>>>,
    pub agent_pool: Option<AgentPool>,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/servers", get(server_info::list).post(servers::create))
        .route("/api/v1/servers/{id}", get(server_info::get_one).put(server_info::update))
        .route("/api/v1/servers/{id}/rcon", post(rcon::execute))
        .route("/api/v1/servers/{id}/rcon-logs", get(rcon::list_logs))
        .route("/api/v1/servers/{id}/logs", get(logs::list))
        .route("/api/v1/servers/{id}/logs/stream", get(logs::stream))
        .route("/api/v1/servers/{id}/files", get(files::read_file).put(files::write_file))
        .route("/api/v1/servers/{id}/files/list", get(files::list_files))
        .route("/agent/connect", get(agent_ws::handler))
        .with_state(state)
}
