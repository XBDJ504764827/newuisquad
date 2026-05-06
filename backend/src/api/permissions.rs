use axum::{Json, extract::{State, Path}, http::StatusCode};
use crate::api::AppState;
use crate::models::permission::{
    CreatePermissionGroupRequest, UpdatePermissionGroupRequest,
    CreatePermissionAdminRequest, UpdatePermissionAdminRequest,
};

// ════════════════════════════════════════════
//  权限组 CRUD
// ════════════════════════════════════════════

/// GET /api/v1/servers/{id}/permission-groups
pub async fn list_groups(
    State(state): State<AppState>, Path(server_id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let groups = sqlx::query_as::<_, crate::models::permission::PermissionGroup>(
        "SELECT * FROM permission_groups WHERE server_id=$1 ORDER BY id"
    ).bind(server_id).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": groups })))
}

/// POST /api/v1/servers/{id}/permission-groups
pub async fn create_group(
    State(state): State<AppState>, Path(server_id): Path<i32>,
    Json(req): Json<CreatePermissionGroupRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if req.group_name.is_empty() {
        return Ok(Json(serde_json::json!({ "error": "组名不能为空" })));
    }
    match sqlx::query_as::<_, crate::models::permission::PermissionGroup>(
        "INSERT INTO permission_groups (server_id, group_name, permissions) VALUES ($1,$2,$3) RETURNING *"
    ).bind(server_id).bind(&req.group_name).bind(&req.permissions).fetch_one(&state.pool).await {
        Ok(g) => Ok(Json(serde_json::json!(g))),
        Err(_) => Ok(Json(serde_json::json!({ "error": "组名已存在" }))),
    }
}

/// PUT /api/v1/servers/{id}/permission-groups/{gid}
pub async fn update_group(
    State(state): State<AppState>,
    Path((server_id, gid)): Path<(i32, i32)>,
    Json(req): Json<UpdatePermissionGroupRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let current = match sqlx::query_as::<_, crate::models::permission::PermissionGroup>(
        "SELECT * FROM permission_groups WHERE id=$1 AND server_id=$2"
    ).bind(gid).bind(server_id).fetch_optional(&state.pool).await {
        Ok(Some(g)) => g,
        _ => return Ok(Json(serde_json::json!({ "error": "组不存在" }))),
    };
    let name = req.group_name.unwrap_or(current.group_name);
    let perms = req.permissions.unwrap_or(current.permissions);
    match sqlx::query_as::<_, crate::models::permission::PermissionGroup>(
        "UPDATE permission_groups SET group_name=$1, permissions=$2, updated_at=NOW() WHERE id=$3 AND server_id=$4 RETURNING *"
    ).bind(&name).bind(&perms).bind(gid).bind(server_id).fetch_one(&state.pool).await {
        Ok(g) => Ok(Json(serde_json::json!(g))),
        Err(_) => Ok(Json(serde_json::json!({ "error": "更新失败" }))),
    }
}

/// DELETE /api/v1/servers/{id}/permission-groups/{gid}
pub async fn delete_group(
    State(state): State<AppState>,
    Path((server_id, gid)): Path<(i32, i32)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _ = sqlx::query("DELETE FROM permission_groups WHERE id=$1 AND server_id=$2")
        .bind(gid).bind(server_id).execute(&state.pool).await;
    Ok(Json(serde_json::json!({ "success": true })))
}

// ════════════════════════════════════════════
//  管理员 SteamID 映射 CRUD
// ════════════════════════════════════════════

/// GET /api/v1/servers/{id}/permission-admins
pub async fn list_admins(
    State(state): State<AppState>, Path(server_id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let admins = sqlx::query_as::<_, crate::models::permission::PermissionAdmin>(
        "SELECT * FROM permission_admins WHERE server_id=$1 ORDER BY id"
    ).bind(server_id).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": admins })))
}

/// POST /api/v1/servers/{id}/permission-admins
pub async fn create_admin(
    State(state): State<AppState>, Path(server_id): Path<i32>,
    Json(req): Json<CreatePermissionAdminRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if req.steam_id.len() < 10 {
        return Ok(Json(serde_json::json!({ "error": "无效的 SteamID64" })));
    }
    if req.group_name.is_empty() {
        return Ok(Json(serde_json::json!({ "error": "请选择权限组" })));
    }
    match sqlx::query_as::<_, crate::models::permission::PermissionAdmin>(
        "INSERT INTO permission_admins (server_id, steam_id, group_name, player_name) VALUES ($1,$2,$3,$4) RETURNING *"
    ).bind(server_id).bind(&req.steam_id).bind(&req.group_name).bind(&req.player_name).fetch_one(&state.pool).await {
        Ok(a) => Ok(Json(serde_json::json!(a))),
        Err(_) => Ok(Json(serde_json::json!({ "error": "该 SteamID 已存在" }))),
    }
}

/// PUT /api/v1/servers/{id}/permission-admins/{aid}
pub async fn update_admin(
    State(state): State<AppState>,
    Path((server_id, aid)): Path<(i32, i32)>,
    Json(req): Json<UpdatePermissionAdminRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let current = match sqlx::query_as::<_, crate::models::permission::PermissionAdmin>(
        "SELECT * FROM permission_admins WHERE id=$1 AND server_id=$2"
    ).bind(aid).bind(server_id).fetch_optional(&state.pool).await {
        Ok(Some(a)) => a,
        _ => return Ok(Json(serde_json::json!({ "error": "记录不存在" }))),
    };
    let group = req.group_name.unwrap_or(current.group_name);
    let name = req.player_name.unwrap_or(current.player_name);
    match sqlx::query_as::<_, crate::models::permission::PermissionAdmin>(
        "UPDATE permission_admins SET group_name=$1, player_name=$2 WHERE id=$3 AND server_id=$4 RETURNING *"
    ).bind(&group).bind(&name).bind(aid).bind(server_id).fetch_one(&state.pool).await {
        Ok(a) => Ok(Json(serde_json::json!(a))),
        Err(_) => Ok(Json(serde_json::json!({ "error": "更新失败" }))),
    }
}

/// DELETE /api/v1/servers/{id}/permission-admins/{aid}
pub async fn delete_admin(
    State(state): State<AppState>,
    Path((server_id, aid)): Path<(i32, i32)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _ = sqlx::query("DELETE FROM permission_admins WHERE id=$1 AND server_id=$2")
        .bind(aid).bind(server_id).execute(&state.pool).await;
    Ok(Json(serde_json::json!({ "success": true })))
}

// ════════════════════════════════════════════
//  公开 .cfg 端点（无需认证，供游戏服务器下载）
// ════════════════════════════════════════════

const ALL_PERMISSIONS: &[&str] = &[
    "reserve", "balance", "canseeadminchat", "manageserver", "teamchange",
    "chat", "cameraman", "kick", "ban", "forceteamchange", "immune",
    "changemap", "pause", "cheat", "private", "config", "featuretest",
    "demos", "disbandSquad", "removeFromSquad", "demoteCommander", "debug",
];

/// GET /api/v1/servers/{id}/Admins.cfg — 公开，无需认证
pub async fn serve_admins_cfg(
    State(state): State<AppState>, Path(server_id): Path<i32>,
) -> Result<(StatusCode, [(String, String); 2], String), StatusCode> {
    let groups = sqlx::query_as::<_, crate::models::permission::PermissionGroup>(
        "SELECT * FROM permission_groups WHERE server_id=$1 ORDER BY id"
    ).bind(server_id).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let admins = sqlx::query_as::<_, crate::models::permission::PermissionAdmin>(
        "SELECT * FROM permission_admins WHERE server_id=$1 ORDER BY id"
    ).bind(server_id).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut cfg = String::new();

    // Group 定义
    for g in &groups {
        cfg.push_str(&format!("Group={}:{}\n", g.group_name, g.permissions));
    }
    if !groups.is_empty() { cfg.push('\n'); }

    // Admin 映射
    for a in &admins {
        let comment = if a.player_name.is_empty() { "" } else { &a.player_name };
        cfg.push_str(&format!("Admin={}:{} // {}\n", a.steam_id, a.group_name, comment));
    }
    if cfg.ends_with('\n') { cfg.pop(); }

    let headers = [
        ("Content-Type".to_string(), "text/plain; charset=utf-8".to_string()),
        ("X-Content-Type-Options".to_string(), "nosniff".to_string()),
    ];
    Ok((StatusCode::OK, headers, cfg))
}

/// GET /api/v1/servers/{id}/Bans.cfg — 公开，无需认证
/// 格式: SteamID64:Duration //Reason 处理人：AdminName
pub async fn serve_bans_cfg(
    State(state): State<AppState>, Path(server_id): Path<i32>,
) -> Result<(StatusCode, [(String, String); 2], String), StatusCode> {
    let bans = sqlx::query_as::<_, crate::models::permission::BanRecord>(
        "SELECT * FROM bans WHERE server_id=$1 ORDER BY id"
    ).bind(server_id).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut cfg = String::new();
    for b in &bans {
        let comment = if b.admin_user.is_empty() {
            b.reason.clone()
        } else {
            format!("{} 处理人：{}", b.reason, b.admin_user)
        };
        cfg.push_str(&format!("{}:{} //{}\n", b.steam_id, b.duration, comment));
    }
    if cfg.ends_with('\n') { cfg.pop(); }

    let headers = [
        ("Content-Type".to_string(), "text/plain; charset=utf-8".to_string()),
        ("X-Content-Type-Options".to_string(), "nosniff".to_string()),
    ];
    Ok((StatusCode::OK, headers, cfg))
}
