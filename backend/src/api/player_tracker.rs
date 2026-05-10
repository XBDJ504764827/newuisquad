use axum::{Json, extract::{State, Path, Query}, http::StatusCode};
use serde::Deserialize;
use crate::api::AppState;
use crate::services::player_tracker;

#[derive(Deserialize, Default)]
pub struct SearchQuery {
    pub name: Option<String>,
    pub steam_id: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Deserialize, Default)]
pub struct PageQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

// ════════════════════════════════════════════
//  实时玩家状态 API
// ════════════════════════════════════════════

/// GET /api/v1/servers/{id}/live-state
/// 从 PlayerTracker 缓存获取实时玩家/小队/队伍状态
pub async fn live_state(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some(ref tracker) = state.game_services.player_tracker {
        if let Some(s) = tracker.get_state(server_id).await {
            return Ok(Json(serde_json::json!(s)));
        }
    }
    // 回退：从 Agent 缓存获取
    let cache = state.server_states.read().await;
    if let Some(val) = cache.get(&server_id.to_string()) {
        return Ok(Json(val.clone()));
    }
    Ok(Json(serde_json::json!({ "error": "无数据" })))
}

/// GET /api/v1/servers/{id}/live-players
pub async fn live_players(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some(ref tracker) = state.game_services.player_tracker {
        if let Some(s) = tracker.get_state(server_id).await {
            let players: Vec<&player_tracker::TrackedPlayer> = if let Some(ref name) = q.name {
                let lower = name.to_lowercase();
                s.players.iter().filter(|p| p.name.to_lowercase().contains(&lower)).collect()
            } else if let Some(ref sid) = q.steam_id {
                s.players.iter().filter(|p| p.steam_id == *sid).collect()
            } else {
                s.players.iter().collect()
            };
            return Ok(Json(serde_json::json!({ "data": players, "total": players.len() })));
        }
    }
    Ok(Json(serde_json::json!({ "data": [], "total": 0 })))
}

/// GET /api/v1/servers/{id}/live-squads
pub async fn live_squads(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some(ref tracker) = state.game_services.player_tracker {
        if let Some(s) = tracker.get_state(server_id).await {
            return Ok(Json(serde_json::json!({ "data": s.squads, "total": s.squads.len() })));
        }
    }
    Ok(Json(serde_json::json!({ "data": [], "total": 0 })))
}

/// GET /api/v1/servers/{id}/live-teams
pub async fn live_teams(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some(ref tracker) = state.game_services.player_tracker {
        if let Some(s) = tracker.get_state(server_id).await {
            return Ok(Json(serde_json::json!({ "data": s.teams, "total": s.teams.len() })));
        }
    }
    Ok(Json(serde_json::json!({ "data": [], "total": 0 })))
}

/// GET /api/v1/servers/{id}/live-team-players/{team_id}
pub async fn live_team_players(
    State(state): State<AppState>,
    Path((server_id, team_id)): Path<(i32, i32)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some(ref tracker) = state.game_services.player_tracker {
        let players = tracker.get_players_by_team(server_id, team_id).await;
        return Ok(Json(serde_json::json!({ "data": players, "total": players.len() })));
    }
    Ok(Json(serde_json::json!({ "data": [], "total": 0 })))
}

/// GET /api/v1/servers/{id}/live-squad-players/{squad_id}
pub async fn live_squad_players(
    State(state): State<AppState>,
    Path((server_id, squad_id)): Path<(i32, String)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some(ref tracker) = state.game_services.player_tracker {
        let players = tracker.get_players_by_squad(server_id, &squad_id).await;
        return Ok(Json(serde_json::json!({ "data": players, "total": players.len() })));
    }
    Ok(Json(serde_json::json!({ "data": [], "total": 0 })))
}

/// POST /api/v1/servers/{id}/live-refresh
/// 手动触发该服务器的玩家状态刷新
pub async fn trigger_refresh(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some(ref tracker) = state.game_services.player_tracker {
        tracker.force_refresh(server_id).await;
        return Ok(Json(serde_json::json!({ "success": true, "message": "已触发刷新" })));
    }
    Ok(Json(serde_json::json!({ "error": "PlayerTracker 未启用" })))
}

/// GET /api/v1/players/search
/// 跨服务器搜索玩家
pub async fn search_players(
    State(state): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some(ref tracker) = state.game_services.player_tracker {
        if let Some(ref name) = q.name {
            let results = tracker.find_player_by_name(name).await;
            let data: Vec<serde_json::Value> = results.into_iter().map(|(sid, p)| {
                serde_json::json!({ "server_id": sid, "player": p })
            }).collect();
            return Ok(Json(serde_json::json!({ "data": data, "total": data.len() })));
        }
        if let Some(ref sid) = q.steam_id {
            if let Some((server_id, p)) = tracker.find_player_by_steam(sid).await {
                return Ok(Json(serde_json::json!({ "data": [{ "server_id": server_id, "player": p }], "total": 1 })));
            }
        }
    }
    Ok(Json(serde_json::json!({ "data": [], "total": 0 })))
}

// ════════════════════════════════════════════
//  增强版比赛 API
// ════════════════════════════════════════════

/// GET /api/v1/servers/{id}/match-summaries
/// 增强版比赛摘要（含击杀/TK统计、链式ID、时长）
pub async fn match_summaries(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Query(q): Query<PageQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(20).min(200);

    match player_tracker::get_match_summaries(&state.pool, server_id, page, per_page).await {
        Ok((summaries, total)) => Ok(Json(serde_json::json!({
            "data": summaries, "total": total, "page": page, "per_page": per_page
        }))),
        Err(e) => Ok(Json(serde_json::json!({ "error": format!("查询失败: {}", e) }))),
    }
}

/// GET /api/v1/servers/{id}/match-players/{match_id}
/// 单场比赛的玩家统计
pub async fn match_player_stats(
    State(state): State<AppState>,
    Path((server_id, match_id)): Path<(i32, i32)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match player_tracker::get_match_player_stats(&state.pool, server_id, match_id).await {
        Ok(stats) => Ok(Json(serde_json::json!({ "data": stats, "total": stats.len() }))),
        Err(e) => Ok(Json(serde_json::json!({ "error": format!("查询失败: {}", e) }))),
    }
}
