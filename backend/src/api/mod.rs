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
pub mod auth_middleware;
pub mod chat;
pub mod squadjs_report;
pub mod rate_limiter;
pub mod team_switch;
pub mod bans;
pub mod permissions;
pub mod player_tracker;
pub mod chat_moderation;
pub mod server_health;
pub mod game_services;
pub mod audit_config;
pub mod player_profile;
pub mod workflows;

use axum::{Router, routing::{get, post, put}};
use axum::middleware::from_fn;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;
use crate::models::server_log::LogEntry;
use crate::api::agent_ws::AgentPool;
use crate::api::rate_limiter::RateLimiterState;
use crate::api::auth_middleware::CacheEntry;
use crate::services::log_batcher::LogBatcher;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub log_broadcast: Option<Arc<broadcast::Sender<LogEntry>>>,
    pub agent_pool: Option<AgentPool>,
    pub steam_api_key: String,
    pub jwt_secret: String,
    // Agent 上报的服务器状态缓存 { server_id -> state_json }
    pub server_states: Arc<tokio::sync::RwLock<std::collections::HashMap<String, serde_json::Value>>>,
    // 代码跳边开关缓存: server_id -> enabled
    pub team_switch_cache: Arc<tokio::sync::RwLock<std::collections::HashMap<i32, bool>>>,
    // 批量日志写入器
    pub log_batcher: LogBatcher,
    pub rate_limiter: RateLimiterState,
    // RCON 连接池
    pub rcon_pool: crate::rcon_client::pool::RconPool,
    // 实时玩家追踪
    pub player_tracker: Option<Arc<crate::services::player_tracker::PlayerTracker>>,
    // 聊天审核
    pub chat_automod: Option<Arc<tokio::sync::RwLock<crate::services::chat_automod::ChatAutomod>>>,
    // 服务器健康监控
    pub server_monitor: Option<Arc<crate::services::server_monitor::ServerMonitor>>,
    // 播种模式
    pub seeding_service: Option<Arc<crate::services::seeding_service::SeedModeService>>,
    // 队伍平衡
    pub team_balance: Option<Arc<crate::services::team_balance_service::TeamBalanceService>>,
    pub afk_service: Option<Arc<crate::services::afk_service::AfkService>>,
    pub event_manager: Option<Arc<crate::services::event_manager::EventManager>>,
    // 权限版本号缓存 (username -> CacheEntry)
    pub permission_version_cache: Arc<tokio::sync::RwLock<std::collections::HashMap<String, CacheEntry>>>,
}

pub fn build_router(state: AppState) -> Router {
    // 登录路由（更严格的限流）
    let login = Router::new()
        .route("/api/v1/auth/login", post(auth::login))
        .layer(from_fn(rate_limiter::login_rate_limit));

    // 公开路由（无需认证）
    let public = Router::new()
        .merge(login)
        .route("/api/v1/auth/verify", post(auth::verify_token))
        .route("/agent/connect", get(agent_ws::handler))
        .route("/api/v1/servers/{id}/squadjs/update", post(squadjs_report::handler))
        .route("/api/v1/servers/{id}/Admins.cfg", get(permissions::serve_admins_cfg))
        .route("/api/v1/servers/{id}/Bans.cfg", get(permissions::serve_bans_cfg));

    // 受保护路由（需要 Bearer Token 认证 + 通用限流）
    let protected = Router::new()
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
        .route("/api/v1/servers/{id}/team-switch-config", get(team_switch::get_config).put(team_switch::update_config))
        .route("/api/v1/servers/{id}/fly-events", get(squad_events::fly_events))
        .route("/api/v1/servers/{id}/kill-events", get(squad_events::kill_events))
        .route("/api/v1/servers/{id}/match-events", get(squad_events::match_events))
        .route("/api/v1/servers/{id}/explosion-events", get(squad_events::explosion_events))
        .route("/api/v1/servers/{id}/summary", get(server_info::summary))
        .route("/api/v1/servers/{id}/player-info", get(squad_events::player_info))
        .route("/api/v1/servers/{id}/deployable-events", get(squad_events::deployable_events))
        .route("/api/v1/servers/{id}/tick-rate-events", get(squad_events::tick_rate_events))
        .route("/api/v1/servers/{id}/vehicle-events", get(squad_events::vehicle_events))
        .route("/api/v1/servers/{id}/admin-broadcasts", get(squad_events::admin_broadcasts))
        .route("/api/v1/servers/{id}/chat-messages", get(chat::list))
        .route("/api/v1/servers/{id}/server-state", get(server_control::get_server_state))
        .route("/api/v1/servers/{id}/server-info", get(server_control::get_server_info))
        .route("/api/v1/servers/{id}/bans", get(server_control::get_bans))
        .route("/api/v1/servers/{id}/warns", get(server_control::get_warns))
        .route("/api/v1/servers/{id}/player-action", post(server_control::player_action))
        .route("/api/v1/servers/{id}/ban-list", get(bans::ban_list))
        .route("/api/v1/servers/{id}/ban-player", post(bans::ban_player))
        .route("/api/v1/steam-player/{steam_id}", get(bans::steam_player_lookup))
        .route("/api/v1/servers/{id}/permission-groups", get(permissions::list_groups).post(permissions::create_group))
        .route("/api/v1/servers/{id}/permission-groups/{gid}", put(permissions::update_group).delete(permissions::delete_group))
        .route("/api/v1/servers/{id}/permission-admins", get(permissions::list_admins).post(permissions::create_admin))
        .route("/api/v1/servers/{id}/permission-admins/{aid}", put(permissions::update_admin).delete(permissions::delete_admin))
        .route("/api/v1/servers/{id}/permission-resolve/{group}", get(permissions::resolve_permissions))
        .route("/api/v1/permission-catalog", get(permissions::permission_catalog))
        .route("/api/v1/servers/{id}/permission-groups/{gid}/copy-from-template", post(permissions::copy_from_template))
        .route("/api/v1/servers/{id}/disband-squad/{team_id}/{squad_id}", axum::routing::delete(server_control::disband_squad))
        .route("/api/v1/operation-logs", get(operation_logs::list))
        .route("/api/v1/players/search", get(player_tracker::search_players))
        .route("/api/v1/servers/{id}/live-state", get(player_tracker::live_state))
        .route("/api/v1/servers/{id}/live-players", get(player_tracker::live_players))
        .route("/api/v1/servers/{id}/live-squads", get(player_tracker::live_squads))
        .route("/api/v1/servers/{id}/live-teams", get(player_tracker::live_teams))
        .route("/api/v1/servers/{id}/live-team-players/{team_id}", get(player_tracker::live_team_players))
        .route("/api/v1/servers/{id}/live-squad-players/{squad_id}", get(player_tracker::live_squad_players))
        .route("/api/v1/servers/{id}/live-refresh", post(player_tracker::trigger_refresh))
        .route("/api/v1/servers/{id}/match-summaries", get(player_tracker::match_summaries))
        .route("/api/v1/servers/{id}/match-players/{match_id}", get(player_tracker::match_player_stats))
        .route("/api/v1/servers/{id}/chat-moderation-settings", get(chat_moderation::get_settings).put(chat_moderation::update_settings))
        .route("/api/v1/servers/{id}/chat-violations", get(chat_moderation::list_violations))
        .route("/api/v1/servers/enhanced", get(server_health::list_enhanced))
        .route("/api/v1/servers-health", get(server_health::all_health))
        .route("/api/v1/servers/{id}/health", get(server_health::server_health))
        .route("/api/v1/servers/{id}/stats", get(server_health::server_stats))
        .route("/api/v1/servers/{id}/seed-status", get(game_services::seed_status))
        .route("/api/v1/servers/{id}/team-balance-status", get(game_services::balance_status))
        .route("/api/v1/servers/{id}/scramble", post(game_services::manual_scramble))
        .route("/api/v1/audit-stats", get(audit_config::audit_stats))
        .route("/api/v1/audit-detail", get(audit_config::audit_detail))
        .route("/api/v1/servers/{id}/config-history", get(audit_config::config_history))
        .route("/api/v1/servers/{id}/config-history/{version}", get(audit_config::config_history_detail))
        .route("/api/v1/command-catalog", get(game_services::command_catalog_list))
        .route("/api/v1/motd/preview", post(game_services::motd_preview))
        .route("/api/v1/servers/{id}/motd-preview", get(game_services::server_motd_preview))
        .route("/api/v1/identity/compute", post(game_services::compute_identities))
        .route("/api/v1/identity/lookup", get(game_services::identity_lookup))
        .route("/api/v1/identities", get(game_services::identity_list))
        .route("/api/v1/player-profile/{steam64}", get(player_profile::get_profile))
        .route("/api/v1/servers/{id}/workflows", get(workflows::list).post(workflows::create))
        .route("/api/v1/servers/{id}/workflows/{wid}", get(workflows::get_one).put(workflows::update).delete(workflows::delete))
        .route("/api/v1/servers/{id}/workflows/{wid}/toggle", post(workflows::toggle))
        .route("/api/v1/servers/{id}/workflows/{wid}/executions", get(workflows::executions))
        .route("/api/v1/admins", get(admin_users::list).post(admin_users::create))
        .route("/api/v1/admins/{id}", put(admin_users::update).delete(admin_users::delete))
        .layer(from_fn(rate_limiter::rate_limit))
        .layer(from_fn(auth_middleware::require_auth));

    Router::new()
        .merge(public)
        .merge(protected)
        .with_state(state.clone())
        .layer(axum::Extension(state.rate_limiter.clone()))
        .layer(axum::Extension(state.jwt_secret.clone()))
        .layer(axum::Extension(state.clone()))
}
