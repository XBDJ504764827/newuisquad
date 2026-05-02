use axum::{Json, extract::{State, Path}, http::StatusCode};
use serde::Deserialize;
use crate::api::AppState;
use crate::services::rcon_server_info;

#[derive(Deserialize)]
pub struct PlayerAction {
    pub player_name: String,
    pub action: String, // warn, kick, ban, team_change, squad_remove
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub admin_user: String,
}

/// 获取服务器实时状态
pub async fn get_server_state(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let row = sqlx::query_as::<_, (String, i32, String)>(
        "SELECT ip, rcon_port, rcon_password FROM servers WHERE id=$1"
    ).bind(server_id).fetch_optional(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (ip, port, password) = match row {
        Some(r) => r,
        None => return Ok(Json(serde_json::json!({ "error": "服务器不存在" }))),
    };

    match rcon_server_info::get_server_state(&ip, port as u16, &password).await {
        Ok(state) => {
            Ok(Json(serde_json::json!({
                "players": state.players.iter().map(|p| serde_json::json!({
                    "name": p.name, "steam_id": p.steam_id, "team_id": p.team_id,
                    "squad_id": p.squad_id, "role": p.role,
                    "kills": p.kills, "deaths": p.deaths, "score": p.score, "ping": p.ping,
                    "is_admin": p.is_admin,
                })).collect::<Vec<_>>(),
                "squads": state.squads.iter().map(|s| serde_json::json!({
                    "name": s.name, "creator": s.creator, "team_id": s.team_id,
                })).collect::<Vec<_>>(),
                "teams": state.teams,
                "map_name": state.map_name,
                "game_mode": state.game_mode,
            })))
        }
        Err(e) => Ok(Json(serde_json::json!({ "error": e }))),
    }
}

/// 执行玩家操作
pub async fn player_action(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Json(req): Json<PlayerAction>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let row = sqlx::query_as::<_, (String, i32, String)>(
        "SELECT ip, rcon_port, rcon_password FROM servers WHERE id=$1"
    ).bind(server_id).fetch_optional(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (ip, port, password) = match row {
        Some(r) => r,
        None => return Ok(Json(serde_json::json!({ "error": "服务器不存在" }))),
    };

    let mut rcon = match crate::rcon_client::squad::SquadRcon::connect(&ip, port as u16, &password).await {
        Ok(r) => r,
        Err(e) => return Ok(Json(serde_json::json!({ "error": format!("RCON 连接失败: {}", e) }))),
    };

    let cmd = match req.action.as_str() {
        "warn" => format!("AdminWarn \"{}\" \"{}\"", req.player_name, if req.message.is_empty() { "管理员警告" } else { &req.message }),
        "kick" => format!("AdminKick \"{}\" \"{}\"", req.player_name, if req.message.is_empty() { "被管理员踢出" } else { &req.message }),
        "ban" => format!("AdminBan \"{}\" \"{}\"", req.player_name, if req.message.is_empty() { "被管理员封禁" } else { &req.message }),
        "team_change" => format!("AdminForceTeamChange \"{}\"", req.player_name),
        "squad_remove" => format!("AdminRemoveFromSquad \"{}\"", req.player_name),
        _ => return Ok(Json(serde_json::json!({ "error": "未知操作" }))),
    };

    match rcon.execute(&cmd).await {
        Ok(resp) => {
            tracing::info!(server_id, admin = %req.admin_user, action = %req.action, target = %req.player_name, "玩家操作执行成功");
            Ok(Json(serde_json::json!({ "success": true, "response": resp })))
        }
        Err(e) => Ok(Json(serde_json::json!({ "error": format!("执行失败: {}", e) }))),
    }
}

/// 解散小队
pub async fn disband_squad(
    State(state): State<AppState>,
    Path((server_id, team_id, squad_id)): Path<(i32, i32, String)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let row = sqlx::query_as::<_, (String, i32, String)>(
        "SELECT ip, rcon_port, rcon_password FROM servers WHERE id=$1"
    ).bind(server_id).fetch_optional(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (ip, port, password) = match row {
        Some(r) => r,
        None => return Ok(Json(serde_json::json!({ "error": "服务器不存在" }))),
    };

    let mut rcon = match crate::rcon_client::squad::SquadRcon::connect(&ip, port as u16, &password).await {
        Ok(r) => r,
        Err(e) => return Ok(Json(serde_json::json!({ "error": format!("RCON 连接失败: {}", e) }))),
    };

    let cmd = format!("AdminDisbandSquad {} {}", team_id, squad_id);
    match rcon.execute(&cmd).await {
        Ok(resp) => Ok(Json(serde_json::json!({ "success": true, "response": resp }))),
        Err(e) => Ok(Json(serde_json::json!({ "error": format!("执行失败: {}", e) }))),
    }
}
