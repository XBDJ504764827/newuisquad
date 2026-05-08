use axum::{Json, extract::{State, Path, Query}, http::StatusCode};
use serde::Deserialize;
use uuid::Uuid;
use crate::api::AppState;
use crate::protocol::AgentMessage;

/// 合并后的封禁条目
#[derive(serde::Serialize)]
pub struct MergedBanEntry {
    pub steam_id: String,
    pub player_name: String,
    pub duration: String,  // "permanent" or minutes
    pub reason: String,
    pub source: String,    // "RCON" or "ban.cfg"
}

#[derive(Deserialize)]
pub struct BanPlayerRequest {
    pub steam_id: String,
    pub reason: String,
    #[serde(default)]
    pub duration: i32,  // 0=permanent, >0=minutes
}

#[derive(Deserialize)]
pub struct BanPlayerQuery {
    pub admin_user: Option<String>,
}

/// GET /api/v1/steam-player/{steam_id} — 通过 Steam API 查询玩家名称
pub async fn steam_player_lookup(
    State(state): State<AppState>,
    Path(steam_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if state.steam_api_key.is_empty() {
        return Ok(Json(serde_json::json!({ "error": "未配置 Steam API Key" })));
    }
    let names = crate::services::steam_service::fetch_player_names(
        &state.steam_api_key,
        &[steam_id.clone()],
    ).await;
    let player_name = names.get(&steam_id).cloned().unwrap_or_default();
    Ok(Json(serde_json::json!({ "steam_id": steam_id, "player_name": player_name })))
}

/// 解析 ban.cfg 内容（兼容两种格式）:
/// 新格式: SteamID64:Duration //Reason 处理人：Admin
/// 旧格式: Ban=SteamID64:Duration:Reason
fn parse_ban_cfg(content: &str) -> Vec<MergedBanEntry> {
    content.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") || line.starts_with('#') { return None; }

            // 新格式: SteamID64:Duration //Comment
            if !line.starts_with("Ban=") {
                let (data, comment) = if let Some(pos) = line.find(" //") {
                    (line[..pos].trim(), line[pos + 3..].trim())
                } else {
                    (line, "")
                };
                let mut parts = data.splitn(2, ':');
                let steam_id = parts.next()?.to_string();
                let duration_str = parts.next()?.to_string();
                if steam_id.len() < 10 { return None; }
                let duration = match duration_str.parse::<i32>() {
                    Ok(0) => "永久封禁".to_string(),
                    Ok(mins) => format!("{}分钟", mins),
                    Err(_) => "永久封禁".to_string(),
                };
                let reason = if comment.is_empty() { "".to_string() } else { comment.to_string() };
                Some(MergedBanEntry {
                    steam_id,
                    player_name: String::new(),
                    duration,
                    reason,
                    source: "ban.cfg".to_string(),
                })
            } else {
                // 旧格式: Ban=SteamID64:Duration:Reason
                let rest = &line[4..];
                let mut parts = rest.splitn(3, ':');
                let steam_id = parts.next()?.to_string();
                let duration_str = parts.next()?.to_string();
                let reason = parts.next().unwrap_or("").to_string();
                if steam_id.is_empty() || steam_id.len() < 10 { return None; }
                let duration = match duration_str.parse::<i32>() {
                    Ok(0) => "永久封禁".to_string(),
                    Ok(mins) => format!("{}分钟", mins),
                    Err(_) => "永久封禁".to_string(),
                };
                Some(MergedBanEntry {
                    steam_id,
                    player_name: String::new(),
                    duration,
                    reason,
                    source: "ban.cfg".to_string(),
                })
            }
        })
        .collect()
}

/// GET /api/v1/servers/{id}/ban-list — 合并 RCON 封禁 + ban.cfg 封禁
pub async fn ban_list(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut bans: Vec<MergedBanEntry> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // 1. 获取 RCON 远程封禁列表
    if let Ok(Some((ip, rcon_port, rcon_pass))) = sqlx::query_as::<_, (String, i32, String)>(
        "SELECT ip, rcon_port, rcon_password FROM servers WHERE id=$1"
    ).bind(server_id).fetch_optional(&state.pool).await {
        if let Ok(raw) = state.rcon_pool.execute(&ip, rcon_port as u16, &rcon_pass, "AdminListBans").await {
                for entry in crate::services::rcon_server_info::parse_ban_list(&raw) {
                    if !entry.steam_id.is_empty() && seen.insert(entry.steam_id.clone()) {
                        bans.push(MergedBanEntry {
                            steam_id: entry.steam_id,
                            player_name: entry.player_name,
                            duration: entry.duration,
                            reason: entry.reason,
                            source: "RCON".to_string(),
                        });
                    }
                }
            }
    }

    // 2. 从 bans 表（DB 存储的封禁记录）
    if let Ok(db_bans) = sqlx::query_as::<_, crate::models::permission::BanRecord>(
        "SELECT * FROM bans WHERE server_id=$1 ORDER BY id"
    ).bind(server_id).fetch_all(&state.pool).await {
        for b in db_bans {
            if seen.insert(b.steam_id.clone()) {
                bans.push(MergedBanEntry {
                    steam_id: b.steam_id,
                    player_name: b.player_name,
                    duration: if b.duration == 0 { "永久封禁".to_string() } else { format!("{}分钟", b.duration) },
                    reason: b.reason,
                    source: "DB".to_string(),
                });
            }
        }
    }

    // 3. 读取 ban.cfg
    if let Some(ref agent_pool) = state.agent_pool {
        for path in &["SquadGame/ServerConfig/Bans.cfg", "Bans.cfg", "ban.cfg"] {
            let request_id = Uuid::new_v4().to_string();
            let cmd = AgentMessage::ReadFile {
                request_id: request_id.clone(),
                path: path.to_string(),
            };
            if let Ok(AgentMessage::FileReadResult { content, error, .. }) = agent_pool
                .send_and_wait(&server_id.to_string(), cmd, &request_id).await
            {
                if error.is_none() {
                    if let Some(content) = content {
                        for cfg_ban in parse_ban_cfg(&content) {
                            if seen.insert(cfg_ban.steam_id.clone()) {
                                bans.push(cfg_ban);
                            }
                        }
                    }
                    break;
                }
            }
        }
    }

    // 3. 用 Steam API 补全 ban.cfg 中缺失的玩家名
    let missing_names: Vec<String> = bans.iter()
        .filter(|b| b.player_name.is_empty())
        .map(|b| b.steam_id.clone())
        .collect();
    if !missing_names.is_empty() && !state.steam_api_key.is_empty() {
        let names = crate::services::steam_service::fetch_player_names(&state.steam_api_key, &missing_names).await;
        for ban in bans.iter_mut() {
            if ban.player_name.is_empty() {
                if let Some(name) = names.get(&ban.steam_id) {
                    ban.player_name = name.clone();
                }
            }
        }
    }

    Ok(Json(serde_json::json!({ "data": bans })))
}

/// POST /api/v1/servers/{id}/ban-player — 手动添加封禁
pub async fn ban_player(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Query(query): Query<BanPlayerQuery>,
    Json(req): Json<BanPlayerRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if req.steam_id.len() < 10 || !req.steam_id.chars().all(|c| c.is_ascii_digit()) {
        return Ok(Json(serde_json::json!({ "error": "无效的 SteamID64" })));
    }
    if req.reason.is_empty() {
        return Ok(Json(serde_json::json!({ "error": "请填写封禁理由" })));
    }

    let admin = query.admin_user.unwrap_or_default();
    let duration_str = if req.duration == 0 { "0".to_string() } else { req.duration.to_string() };
    let new_line = format!("{}:{} //{} 处理人：{}", req.steam_id, duration_str, req.reason, admin);

    // 查询 RCON 凭据（后续两步都用）
    let rcon_creds = sqlx::query_as::<_, (String, i32, String)>(
        "SELECT ip, rcon_port, rcon_password FROM servers WHERE id=$1"
    ).bind(server_id).fetch_optional(&state.pool).await.ok().flatten();

    // 1. 先追加写入 ban.cfg（持久化）
    let mut file_updated = false;
    if let Some(ref agent_pool) = state.agent_pool {
        let ban_cfg_path = "SquadGame/ServerConfig/Bans.cfg";
        let request_id = Uuid::new_v4().to_string();

        let existing = if let Ok(AgentMessage::FileReadResult { content, error, .. }) = agent_pool
            .send_and_wait(&server_id.to_string(), AgentMessage::ReadFile {
                request_id: request_id.clone(),
                path: ban_cfg_path.to_string(),
            }, &request_id).await
        {
            if error.is_none() { content.unwrap_or_default() } else { String::new() }
        } else {
            String::new()
        };

        let updated = if existing.is_empty() {
            new_line.clone()
        } else {
            format!("{}\n{}", existing.trim_end(), new_line)
        };

        let write_id = Uuid::new_v4().to_string();
        if let Ok(AgentMessage::FileWriteResult { success, .. }) = agent_pool
            .send_and_wait(&server_id.to_string(), AgentMessage::WriteFile {
                request_id: write_id.clone(),
                path: ban_cfg_path.to_string(),
                content: updated,
            }, &write_id).await
        {
            file_updated = success;
        }
    }

    // 2. 再通过 RCON 在线封禁踢出（即时生效）
    let rcon_result: Option<String> = if file_updated {
        if let Some((ref ip, rcon_port, ref rcon_pass)) = rcon_creds {
            let player_id = match crate::services::rcon_server_info::list_players(&state.rcon_pool, ip, rcon_port as u16, rcon_pass).await {
                Ok(players) => {
                    players.iter()
                        .find(|p| p.steam_id == req.steam_id)
                        .map(|p| p.player_id)
                }
                Err(_) => None,
            };

            if let Some(pid) = player_id {
                let cmd = format!("AdminBan {} {} {}", pid, req.duration, req.reason);
                match state.rcon_pool.execute(ip, rcon_port as u16, rcon_pass, &cmd).await {
                    Ok(resp) => {
                        tracing::info!(server_id, player_id = pid, steam_id = %req.steam_id, "RCON AdminBan 踢出成功");
                        Some(resp)
                    }
                    Err(e) => {
                        tracing::warn!(server_id, pid, steam_id = %req.steam_id, error = %e, "RCON AdminBan 失败，ban.cfg 已写入");
                        None
                    }
                }
            } else {
                tracing::info!(server_id, steam_id = %req.steam_id, "玩家不在线，仅写入 ban.cfg");
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // 3. 写入 bans 表（供 HTTP Bans.cfg 服务）
    let player_name = crate::services::steam_service::fetch_player_names(
        &state.steam_api_key, &[req.steam_id.clone()],
    ).await.get(&req.steam_id).cloned().unwrap_or_default();
    let _ = sqlx::query(
        "INSERT INTO bans (server_id, steam_id, player_name, duration, reason, admin_user) VALUES ($1,$2,$3,$4,$5,$6) ON CONFLICT (server_id, steam_id) DO UPDATE SET duration=$4, reason=$5, admin_user=$6, player_name=$3, created_at=NOW()"
    )
    .bind(server_id)
    .bind(&req.steam_id)
    .bind(&player_name)
    .bind(req.duration)
    .bind(&req.reason)
    .bind(&admin)
    .execute(&state.pool)
    .await;

    // 4. 记录操作日志
    let _ = sqlx::query(
        "INSERT INTO admin_actions (server_id, admin_name, action_type, target, message, raw_line, logged_at) VALUES ($1,$2,'ban',$3,$4,$5,NOW())"
    )
    .bind(server_id)
    .bind(&admin)
    .bind(&req.steam_id)
    .bind(&req.reason)
    .bind(&new_line)
    .execute(&state.pool)
    .await;

    // 5. 也获取本机 RCON 封禁列表中的 bans 写入 bans 表（保持同步）
    if let Some((ref ip, rcon_port, ref rcon_pass)) = rcon_creds {
        if let Ok(raw) = state.rcon_pool.execute(ip, rcon_port as u16, rcon_pass, "AdminListBans").await {
                for entry in crate::services::rcon_server_info::parse_ban_list(&raw) {
                    if !entry.steam_id.is_empty() {
                        let _ = sqlx::query(
                            "INSERT INTO bans (server_id, steam_id, player_name, duration, reason, admin_user) VALUES ($1,$2,$3,0,$4,$5) ON CONFLICT (server_id, steam_id) DO NOTHING"
                        )
                        .bind(server_id)
                        .bind(&entry.steam_id)
                        .bind(&entry.player_name)
                        .bind(&entry.reason)
                        .bind(&entry.admin)
                        .execute(&state.pool)
                        .await;
                    }
                }
            }
    }

    if rcon_result.is_some() || file_updated {
        Ok(Json(serde_json::json!({
            "success": true,
            "rcon_result": rcon_result,
            "file_updated": file_updated,
            "message": if file_updated { "封禁已写入 ban.cfg" } else { "封禁已通过 RCON 执行" },
        })))
    } else {
        Ok(Json(serde_json::json!({ "error": "无法执行封禁：Agent 未连接且 RCON 不可用" })))
    }
}
