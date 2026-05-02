pub mod server_info;
pub mod rcon;
pub mod logs;
pub mod files;
pub mod agent_ws;
pub mod servers;
pub mod admin_users;
pub mod tk_settings;
pub mod afk_settings;
pub mod broadcast_settings;
pub mod announcements;
pub mod auto_replies;
pub mod team_settings;
pub mod seed_settings;
pub mod damage_notify;
pub mod abnormal_damage;
pub mod squad_events;
pub mod server_control;
pub mod operation_logs;
pub mod auth;

use axum::{Router, routing::{get, post, put}};
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
    pub steam_api_key: String,
    pub jwt_secret: String,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/servers", get(server_info::list).post(servers::create))
        .route("/api/v1/servers/{id}", get(server_info::get_one).put(server_info::update).delete(server_info::delete))
        .route("/api/v1/servers/{id}/rcon", post(rcon::execute))
        .route("/api/v1/servers/{id}/rcon-logs", get(rcon::list_logs))
        .route("/api/v1/servers/{id}/logs", get(logs::list))
        .route("/api/v1/servers/{id}/logs/stream", get(logs::stream))
        .route("/api/v1/servers/{id}/files", get(files::read_file).put(files::write_file))
        .route("/api/v1/servers/{id}/files/list", get(files::list_files))
        .route("/api/v1/servers/{id}/tk-settings", get(tk_settings::get).put(tk_settings::update))
        .route("/api/v1/servers/{id}/afk-settings", get(afk_settings::get).put(afk_settings::update))
        .route("/api/v1/servers/{id}/broadcast-settings", get(broadcast_settings::get).put(broadcast_settings::update))
        .route("/api/v1/servers/{id}/announcements", get(announcements::list).post(announcements::create))
        .route("/api/v1/servers/{id}/announcements/{aid}", axum::routing::delete(announcements::delete))
        .route("/api/v1/servers/{id}/auto-replies", get(auto_replies::list).post(auto_replies::create))
        .route("/api/v1/servers/{id}/auto-replies/{rid}", axum::routing::delete(auto_replies::delete))
        .route("/api/v1/servers/{id}/team-settings", get(team_settings::get).put(team_settings::update))
        .route("/api/v1/servers/{id}/seed-settings", get(seed_settings::get).put(seed_settings::update))
        .route("/api/v1/servers/{id}/damage-notify-settings", get(damage_notify::get).put(damage_notify::update))
        .route("/api/v1/servers/{id}/abnormal-damage-config", get(abnormal_damage::get_config).put(abnormal_damage::update_config))
        .route("/api/v1/servers/{id}/abnormal-damage-rules", get(abnormal_damage::list_rules).post(abnormal_damage::create_rule))
        .route("/api/v1/servers/{id}/abnormal-damage-rules/{rid}", axum::routing::delete(abnormal_damage::delete_rule))
        .route("/api/v1/servers/{id}/abnormal-damage-logs", get(abnormal_damage::list_logs))
        .route("/api/v1/servers/{id}/fly-events", get(squad_events::fly_events))
        .route("/api/v1/servers/{id}/kill-events", get(squad_events::kill_events))
        .route("/api/v1/servers/{id}/player-info", get(squad_events::player_info))
        .route("/api/v1/servers/{id}/server-state", get(server_control::get_server_state))
        .route("/api/v1/servers/{id}/player-action", post(server_control::player_action))
        .route("/api/v1/servers/{id}/disband-squad/{team_id}/{squad_id}", axum::routing::delete(server_control::disband_squad))
        .route("/api/v1/auth/login", post(auth::login))
        .route("/api/v1/auth/verify", post(auth::verify_token))
        .route("/api/v1/operation-logs", get(operation_logs::list))
        .route("/api/v1/admins", get(admin_users::list).post(admin_users::create))
        .route("/api/v1/admins/{id}", put(admin_users::update).delete(admin_users::delete))
        .route("/agent/connect", get(agent_ws::handler))
        .with_state(state)
}
