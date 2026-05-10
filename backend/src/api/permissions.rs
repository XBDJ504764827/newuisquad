use axum::{Json, extract::{State, Path}, http::StatusCode};
use crate::api::AppState;
use crate::models::permission::{
    CreatePermissionGroupRequest, UpdatePermissionGroupRequest,
    CreatePermissionAdminRequest, UpdatePermissionAdminRequest,
};
use crate::models::permission::PermissionGroupRow;
use crate::models::permission_constants;

// ════════════════════════════════════════════
//  权限组 CRUD（支持角色继承 & 模板）
// ════════════════════════════════════════════

/// GET /api/v1/servers/{id}/permission-groups
pub async fn list_groups(
    State(state): State<AppState>, Path(server_id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let groups = sqlx::query_as::<_, crate::models::permission::PermissionGroup>(
        "SELECT * FROM permission_groups WHERE server_id=$1 ORDER BY is_template, id"
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
    // Validate parent exists and is on same server
    if let Some(parent_id) = req.parent_group_id {
        let parent = sqlx::query_scalar::<_, i32>(
            "SELECT id FROM permission_groups WHERE id=$1 AND server_id=$2"
        ).bind(parent_id).bind(server_id).fetch_optional(&state.pool).await;
        if parent.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.is_none() {
            return Ok(Json(serde_json::json!({ "error": "父组不存在" })));
        }
    }
    match sqlx::query_as::<_, crate::models::permission::PermissionGroup>(
        "INSERT INTO permission_groups (server_id, group_name, permissions, parent_group_id, is_admin, is_template) \
         VALUES ($1,$2,$3,$4,$5,false) RETURNING *"
    ).bind(server_id).bind(&req.group_name).bind(&req.permissions)
     .bind(req.parent_group_id).bind(req.is_admin)
     .fetch_one(&state.pool).await {
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
    let parent = match req.parent_group_id {
        Some(pid) => pid,
        None => current.parent_group_id,
    };
    let is_admin = req.is_admin.unwrap_or(current.is_admin);
    match sqlx::query_as::<_, crate::models::permission::PermissionGroup>(
        "UPDATE permission_groups SET group_name=$1, permissions=$2, parent_group_id=$3, is_admin=$4, updated_at=NOW() \
         WHERE id=$5 AND server_id=$6 RETURNING *"
    ).bind(&name).bind(&perms).bind(parent).bind(is_admin).bind(gid).bind(server_id)
     .fetch_one(&state.pool).await {
        Ok(g) => Ok(Json(serde_json::json!(g))),
        Err(_) => Ok(Json(serde_json::json!({ "error": "更新失败" }))),
    }
}

/// DELETE /api/v1/servers/{id}/permission-groups/{gid}
pub async fn delete_group(
    State(state): State<AppState>,
    Path((server_id, gid)): Path<(i32, i32)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Prevent deleting groups that are parents of others
    let children = sqlx::query_scalar::<_, i32>(
        "SELECT COUNT(*) FROM permission_groups WHERE parent_group_id=$1 AND server_id=$2"
    ).bind(gid).bind(server_id).fetch_one(&state.pool).await.unwrap_or(0);
    if children > 0 {
        return Ok(Json(serde_json::json!({ "error": "该组正被子组继承，请先解除继承关系" })));
    }
    let _ = sqlx::query("DELETE FROM permission_groups WHERE id=$1 AND server_id=$2 AND is_template=false")
        .bind(gid).bind(server_id).execute(&state.pool).await;
    Ok(Json(serde_json::json!({ "success": true })))
}

/// POST /api/v1/servers/{id}/permission-groups/{gid}/copy-from-template
/// 从模板创建权限组
pub async fn copy_from_template(
    State(state): State<AppState>,
    Path((server_id, gid)): Path<(i32, i32)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let template = match sqlx::query_as::<_, crate::models::permission::PermissionGroup>(
        "SELECT * FROM permission_groups WHERE id=$1 AND server_id=$2 AND is_template=true"
    ).bind(gid).bind(server_id).fetch_optional(&state.pool).await {
        Ok(Some(g)) => g,
        _ => return Ok(Json(serde_json::json!({ "error": "模板不存在" }))),
    };
    let new_name = format!("{} (复制)", template.group_name);
    match sqlx::query_as::<_, crate::models::permission::PermissionGroup>(
        "INSERT INTO permission_groups (server_id, group_name, permissions, parent_group_id, is_admin, is_template) \
         VALUES ($1,$2,$3,$4,$5,false) RETURNING *"
    ).bind(server_id).bind(&new_name).bind(&template.permissions)
     .bind(template.parent_group_id).bind(template.is_admin)
     .fetch_one(&state.pool).await {
        Ok(g) => Ok(Json(serde_json::json!(g))),
        Err(_) => Ok(Json(serde_json::json!({ "error": "组名已存在" }))),
    }
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
//  权限解析 & 预览 API
// ════════════════════════════════════════════

/// GET /api/v1/servers/{id}/permission-resolve/{group_name}
/// 解析权限组（含继承）的最终权限列表
pub async fn resolve_permissions(
    State(state): State<AppState>,
    Path((server_id, group_name)): Path<(i32, String)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let groups = load_group_rows(&state.pool, server_id).await?;
    let mut visited = std::collections::HashSet::new();
    let effective = permission_constants::resolve_effective_permissions(&groups, &group_name, &mut visited);

    // Build inheritance chain
    let inherited_from: Vec<String> = groups.iter()
        .filter(|g| g.group_name != group_name)
        .filter(|g| visited.contains(&g.group_name))
        .map(|g| g.group_name.clone())
        .collect();

    Ok(Json(serde_json::json!({
        "group_name": group_name,
        "permissions": effective,
        "inherited_from": inherited_from,
    })))
}

/// GET /api/v1/permission-catalog
/// 返回所有可用权限的分类列表（供前端参考）
pub async fn permission_catalog() -> Result<Json<serde_json::Value>, StatusCode> {
    let ui_perms: Vec<serde_json::Value> = permission_constants::all_ui_permissions()
        .into_iter()
        .map(|(code, desc)| serde_json::json!({ "code": code, "description": desc }))
        .collect();
    let rcon_perms: Vec<serde_json::Value> = permission_constants::all_rcon_permissions()
        .into_iter()
        .map(|(code, desc)| serde_json::json!({ "code": code, "description": desc }))
        .collect();
    Ok(Json(serde_json::json!({
        "ui": ui_perms,
        "rcon": rcon_perms,
    })))
}

// ════════════════════════════════════════════
//  公开 .cfg 端点（无需认证，供游戏服务器下载）
// ════════════════════════════════════════════

/// GET /api/v1/servers/{id}/Admins.cfg — 公开，无需认证
/// 支持细粒度权限自动映射为 Squad 格式
pub async fn serve_admins_cfg(
    State(state): State<AppState>, Path(server_id): Path<i32>,
) -> Result<(StatusCode, [(String, String); 2], String), StatusCode> {
    let groups = sqlx::query_as::<_, crate::models::permission::PermissionGroup>(
        "SELECT * FROM permission_groups WHERE server_id=$1 AND is_template=false ORDER BY id"
    ).bind(server_id).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let admins = sqlx::query_as::<_, crate::models::permission::PermissionAdmin>(
        "SELECT * FROM permission_admins WHERE server_id=$1 ORDER BY id"
    ).bind(server_id).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Load row format for inheritance resolution
    let group_rows = load_group_rows(&state.pool, server_id).await.unwrap_or_default();

    let mut cfg = String::new();

    // Group 定义 — 解析继承后的最终权限，映射为 Squad 格式
    for g in &groups {
        let squad_perms = resolve_squad_permissions(&group_rows, &g.group_name);
        cfg.push_str(&format!("Group={}:{}\n", g.group_name, squad_perms));
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

// ════════════════════════════════════════════
//  内部辅助函数
// ════════════════════════════════════════════

/// 加载服务器所有权限组的原始行数据（用于继承解析）
async fn load_group_rows(
    pool: &sqlx::PgPool, server_id: i32,
) -> Result<Vec<PermissionGroupRow>, StatusCode> {
    let rows = sqlx::query_as::<_, (i32, i32, String, String, Option<i32>, bool, bool)>(
        "SELECT id, server_id, group_name, permissions, parent_group_id, is_admin, is_template \
         FROM permission_groups WHERE server_id=$1"
    ).bind(server_id).fetch_all(pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(rows.into_iter().map(|(id, server_id, group_name, permissions, parent_group_id, is_admin, is_template)| {
        PermissionGroupRow { id, server_id, group_name, permissions, parent_group_id, is_admin, is_template }
    }).collect())
}

/// 解析权限组的最终 Squad 权限字符串（含继承）
fn resolve_squad_permissions(group_rows: &[PermissionGroupRow], group_name: &str) -> String {
    let mut visited = std::collections::HashSet::new();
    let effective = permission_constants::resolve_effective_permissions(group_rows, group_name, &mut visited);

    let mut squad_perms: Vec<String> = Vec::new();
    for perm in &effective {
        // Map granular rcon:* to Squad format
        if let Some(squad) = permission_constants::rcon_to_squad_permission(perm) {
            squad_perms.push(squad.to_string());
        }
        // Also support legacy flat permissions directly
        else if !perm.contains(':') && !perm.contains('*') {
            squad_perms.push(perm.clone());
        }
    }

    squad_perms.sort();
    squad_perms.dedup();

    if squad_perms.is_empty() { String::new() } else { squad_perms.join(",") }
}
