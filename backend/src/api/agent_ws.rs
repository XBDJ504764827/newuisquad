use axum::extract::{ws::{WebSocket, WebSocketUpgrade, Message}, Query, State};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, oneshot, RwLock};
use crate::models::server_log::LogEntry;
use sqlx::Row;
use crate::protocol::AgentMessage;
use crate::api::AppState;

#[derive(Deserialize)]
pub struct AgentQuery {
    pub token: Option<String>,
}

#[derive(Clone)]
pub struct AgentPool {
    agents: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<AgentMessage>>>>,
    log_tx: broadcast::Sender<LogEntry>,
    pending: Arc<RwLock<HashMap<String, oneshot::Sender<AgentMessage>>>>,
}

impl AgentPool {
    pub fn new() -> Self {
        let (log_tx, _) = broadcast::channel::<LogEntry>(1024);
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            log_tx,
            pending: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn log_tx(&self) -> broadcast::Sender<LogEntry> {
        self.log_tx.clone()
    }

    pub async fn send_and_wait(
        &self,
        server_id: &str,
        cmd: AgentMessage,
        request_id: &str,
    ) -> Result<AgentMessage, String> {
        let agents = self.agents.read().await;
        let tx = agents.get(server_id).ok_or("Agent 未连接")?;
        let (resp_tx, resp_rx) = oneshot::channel();
        tx.send(cmd).map_err(|e| format!("发送失败: {}", e))?;
        // 仅在发送成功后注册等待
        self.pending.write().await.insert(request_id.to_string(), resp_tx);
        match tokio::time::timeout(std::time::Duration::from_secs(10), resp_rx).await {
            Ok(Ok(msg)) => Ok(msg),
            Ok(Err(_)) => Err("Agent 断开".to_string()),
            Err(_) => {
                // 超时后清理 pending 条目，防止内存泄漏
                self.pending.write().await.remove(request_id);
                Err("响应超时".to_string())
            }
        }
    }
}

pub async fn handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<AgentQuery>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let pool = state.pool.clone();

    // 优先从 X-Auth-Token 头获取，降级到 query param（兼容旧 agent）
    let token = headers.get("X-Auth-Token")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| q.token.clone().filter(|t| !t.is_empty()))
        .unwrap_or_default();

    let agent_pool = match state.agent_pool.clone() {
        Some(p) => p,
        None => {
            return ws.on_upgrade(|mut socket| async move {
                let _ = socket.send(Message::Text("{\"error\":\"Agent 连接池未初始化\"}".into())).await;
                let _ = socket.close().await;
            });
        }
    };

    // 验证 token
    let result = sqlx::query("SELECT id FROM servers WHERE token = $1")
        .bind(&token)
        .fetch_optional(&pool)
        .await;

    let server_states = state.server_states.clone();
    match result {
        Ok(Some(row)) => {
            let server_id: i32 = row.get(0);
            ws.on_upgrade(move |socket| handle_socket(socket, pool, agent_pool, server_states, server_id.to_string()))
        }
        _ => {
            ws.on_upgrade(|mut socket| async move {
                let _ = socket.send(Message::Text("{\"error\":\"无效的 token\"}".into())).await;
                let _ = socket.close().await;
            })
        }
    }
}

async fn handle_socket(
    socket: WebSocket,
    db_pool: sqlx::PgPool,
    agent_pool: AgentPool,
    server_states: Arc<std::sync::RwLock<std::collections::HashMap<String, serde_json::Value>>>,
    server_id: String,
) {
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<AgentMessage>();
    let sid: i32 = server_id.parse().unwrap_or(0);
    let rcon_cmd_tx = cmd_tx.clone();
    agent_pool
        .agents
        .write()
        .await
        .insert(server_id.clone(), cmd_tx);
    tracing::info!("Agent 已连接: {}", server_id);
    crate::services::system_log::agent_event(&db_pool, "agent_ws", &format!("Agent 已连接 server_id={}", server_id)).await;

    let (mut ws_sender, mut ws_receiver) = socket.split();

    let pool = db_pool.clone();
    let log_tx = agent_pool.log_tx();
    let pending = agent_pool.pending.clone();

    use crate::services::team_switch::TeamSwitchManager;
    let mut recv_task = tokio::spawn(async move {
        let mut ts_manager = TeamSwitchManager::new();
        while let Some(Ok(msg)) = ws_receiver.next().await {
            if let Ok(text) = msg.to_text() {
                if let Ok(agent_msg) = serde_json::from_str::<AgentMessage>(text) {
                    match &agent_msg {
                        AgentMessage::Log { data } => {
                            let entry = LogEntry {
                                server_id: sid,
                                log_level: data.log_level.clone(),
                                category: data.category.clone(),
                                message: data.message.clone(),
                                raw_line: data.raw_line.clone(),
                                logged_at: data.logged_at,
                            };
                            tracing::debug!(server_id = sid, level = %data.log_level, "收到日志: {}", data.message.chars().take(60).collect::<String>());
                            match log_tx.send(entry.clone()) {
                                Ok(n) => tracing::debug!(server_id = sid, receivers = n, "日志已广播"),
                                Err(e) => tracing::error!(server_id = sid, error = %e, "日志广播失败"),
                            }
                            let _ = crate::repositories::server_log_repo::insert_log_entry(
                                &pool, sid, &entry,
                            )
                            .await;

                            // Squad 日志解析：提取结构化事件
                            use crate::services::squad_log_parser::{parse_line, ParsedEvent};
                            if let Some(raw) = &entry.raw_line {
                                if let Some(event) = parse_line(raw) {
                                    match event {
                                        ParsedEvent::PlayerLogin { player_name, eos_id, steam64, ip, logged_at } => {
                                            if !steam64.is_empty() {
                                                let _ = sqlx::query("INSERT INTO player_info (server_id, player_name, steam64, eos_id, ip, first_seen, last_seen) VALUES ($1,$2,$3,$4,$5,$6,$6) ON CONFLICT DO NOTHING").bind(sid).bind(&player_name).bind(&steam64).bind(&eos_id).bind(&ip).bind(logged_at).execute(&pool).await;
                                                let _ = sqlx::query("UPDATE player_info SET player_name=$1, eos_id=$2, ip=$3, last_seen=$4 WHERE server_id=$5 AND steam64=$6").bind(&player_name).bind(&eos_id).bind(&ip).bind(logged_at).bind(sid).bind(&steam64).execute(&pool).await;
                                            }
                                        }
                                        ParsedEvent::FlyEvent { player_name, eos_id, steam64, event_type, logged_at } => {
                                            let _ = sqlx::query("INSERT INTO fly_events (server_id, player_name, eos_id, steam64, event_type, logged_at) VALUES ($1,$2,$3,$4,$5,$6)").bind(sid).bind(&player_name).bind(&eos_id).bind(&steam64).bind(&event_type).bind(logged_at).execute(&pool).await;
                                        }
                                        ParsedEvent::KillEvent { attacker_name, attacker_eos, attacker_steam64, victim_name, damage, weapon, is_kill, is_teamkill, logged_at } => {
                                            let _ = sqlx::query("INSERT INTO kill_events (server_id, attacker_name, attacker_eos, attacker_steam64, victim_name, damage, weapon, is_kill, is_teamkill, logged_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)").bind(sid).bind(&attacker_name).bind(&attacker_eos).bind(&attacker_steam64).bind(&victim_name).bind(damage).bind(&weapon).bind(is_kill).bind(is_teamkill).bind(logged_at).execute(&pool).await;
                                        }
                                        ParsedEvent::TeamAssignment { player_name, steam64, team_id, logged_at } => {
                                            let _ = sqlx::query("INSERT INTO team_assignments (server_id, player_name, steam64, team_id, logged_at) VALUES ($1,$2,$3,$4,$5)").bind(sid).bind(&player_name).bind(&steam64).bind(team_id).bind(logged_at).execute(&pool).await;
                                        }
                                        ParsedEvent::SquadCreation { player_name, steam64, squad_id, squad_name, faction, logged_at } => {
                                            let _ = sqlx::query("INSERT INTO squad_creations (server_id, player_name, steam64, squad_id, squad_name, faction, logged_at) VALUES ($1,$2,$3,$4,$5,$6,$7)").bind(sid).bind(&player_name).bind(&steam64).bind(&squad_id).bind(&squad_name).bind(&faction).bind(logged_at).execute(&pool).await;
                                        }
                                        ParsedEvent::MatchEvent { map_name, layer_name, team1_faction, team2_faction, winner_team, event_type, logged_at } => {
                                            let _ = sqlx::query("INSERT INTO match_info (server_id, map_name, layer_name, team1_faction, team2_faction, winner_team, event_type, logged_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)").bind(sid).bind(&map_name).bind(&layer_name).bind(&team1_faction).bind(&team2_faction).bind(winner_team).bind(&event_type).bind(logged_at).execute(&pool).await;
                                        }
                                        ParsedEvent::DeployRole { player_name, steam64, role, logged_at } => {
                                            let _ = sqlx::query("UPDATE player_info SET player_name=$1 WHERE server_id=$2 AND steam64=$3 AND player_name=''").bind(&player_name).bind(sid).bind(&steam64).execute(&pool).await;
                                        }
                                        ParsedEvent::ReviveEvent { reviver_name, reviver_steam64, revived_name, revived_steam64, logged_at } => {
                                            let _ = sqlx::query("INSERT INTO revive_events (server_id, reviver_name, reviver_steam64, revived_name, revived_steam64, logged_at) VALUES ($1,$2,$3,$4,$5,$6)").bind(sid).bind(&reviver_name).bind(&reviver_steam64).bind(&revived_name).bind(&revived_steam64).bind(logged_at).execute(&pool).await;
                                        }
                                        ParsedEvent::VehicleEvent { player_name, steam64, vehicle_name, event_type, logged_at } => {
                                            let _ = sqlx::query("INSERT INTO vehicle_events (server_id, player_name, steam64, vehicle_name, event_type, logged_at) VALUES ($1,$2,$3,$4,$5,$6)").bind(sid).bind(&player_name).bind(&steam64).bind(&vehicle_name).bind(&event_type).bind(logged_at).execute(&pool).await;
                                        }
                                        ParsedEvent::AdminAction { admin_name, action_type, target, message, raw_line, logged_at } => {
                                            let _ = sqlx::query("INSERT INTO admin_actions (server_id, admin_name, action_type, target, message, raw_line, logged_at) VALUES ($1,$2,$3,$4,$5,$6,$7)").bind(sid).bind(&admin_name).bind(&action_type).bind(&target).bind(&message).bind(&raw_line).bind(logged_at).execute(&pool).await;
                                        }
                                        ParsedEvent::PlayerDeath { player_name, steam64, killer_steam64, weapon, logged_at } => {
                                            let _ = sqlx::query("INSERT INTO kill_events (server_id, attacker_name, attacker_steam64, victim_name, damage, weapon, is_kill, is_teamkill, logged_at) VALUES ($1,'',$2,$3,0,$4,true,false,$5)").bind(sid).bind(&killer_steam64).bind(&player_name).bind(&weapon).bind(logged_at).execute(&pool).await;
                                        }
                                        ParsedEvent::ChatMessage { player_name, steam64, message, channel, logged_at } => {
                                            let _ = sqlx::query("INSERT INTO chat_messages (server_id, player_name, steam64, message, channel, logged_at) VALUES ($1,$2,$3,$4,$5,$6)").bind(sid).bind(&player_name).bind(&steam64).bind(&message).bind(&channel).bind(logged_at).execute(&pool).await;
                                        }
                                        ParsedEvent::ExplosionEvent { pos_x, pos_y, pos_z, damage_causer, damage_instigator, logged_at } => {
                                            let _ = sqlx::query("INSERT INTO explosion_events (server_id, pos_x, pos_y, pos_z, damage_causer, damage_instigator, logged_at) VALUES ($1,$2,$3,$4,$5,$6,$7)").bind(sid).bind(pos_x).bind(pos_y).bind(pos_z).bind(&damage_causer).bind(&damage_instigator).bind(logged_at).execute(&pool).await;
                                        }
                                        _ => {}
                                    }
                                }
                            }

                            // 代码跳边：处理公共聊天
                            if let Some(category) = &data.category {
                                let is_public_chat = category == "Chat-All" || category == "Chat" || category == "ChatTeam" || category == "ChatSquad";
                                if is_public_chat {
                                    let parsed = data.message.split_once(": ");
                                    if let Some((chat_player, chat_msg)) = parsed {
                                        let cache = server_states.read().ok();
                                        let state = cache.as_ref().and_then(|c| c.get(&sid.to_string()));
                                        tracing::debug!(server_id = %sid, player = %chat_player, msg = %chat_msg, has_state = state.is_some(), "代码跳边: 处理聊天");
                                        let commands = ts_manager.process_chat(
                                            &sid.to_string(),
                                            chat_player,
                                            chat_msg,
                                            state,
                                        );
                                        drop(cache);
                                        if commands.is_empty() {
                                            tracing::warn!(server_id = %sid, player = %chat_player, "代码跳边: 未生成任何命令");
                                        }
                                        for cmd in commands {
                                            tracing::info!(server_id = %sid, player = %chat_player, "代码跳边: {}", cmd);
                                            let _ = rcon_cmd_tx.send(AgentMessage::SendRcon { command: cmd });
                                        }
                                    } else {
                                        tracing::debug!(server_id = %sid, category = ?data.category, msg = %data.message, "聊天消息格式无法解析");
                                    }
                                }
                            }
                        }
                        AgentMessage::FileReadResult { request_id, .. }
                        | AgentMessage::FileWriteResult { request_id, .. }
                        | AgentMessage::FileListResult { request_id, .. } => {
                            let rid = request_id.clone();
                            if let Some(tx) = pending.write().await.remove(&rid) {
                                let _ = tx.send(agent_msg.clone());
                            }
                        }
                        AgentMessage::ServerStateReport { players, squads, team_names, map_name, game_mode, server_name, player_count, max_players, next_map } => {
                            tracing::info!(server_id = %sid, players = players.len(), squads = squads.len(), map = %map_name, "收到服务器状态上报");
                            let teams: Vec<serde_json::Value> = team_names.iter().map(|t| serde_json::json!({
                                "team_id": t.team_id, "faction": t.faction,
                            })).collect();

                            // 从玩家列表交叉参照找出小队长：is_leader=true 且有所属小队
                            let squad_leaders: Vec<serde_json::Value> = players.iter()
                                .filter(|p| p.is_leader)
                                .filter_map(|p| {
                                    p.squad_id.as_ref().map(|sid| serde_json::json!({
                                        "name": p.name,
                                        "steam_id": p.steam_id,
                                        "team_id": p.team_id,
                                        "squad_id": sid,
                                    }))
                                })
                                .collect();

                            let leader_map: std::collections::HashMap<String, &serde_json::Value> = squad_leaders.iter()
                                .filter_map(|l| {
                                    l["squad_id"].as_str().map(|sid| (sid.to_string(), l))
                                })
                                .collect();

                            let state = serde_json::json!({
                                "players": players.iter().map(|p| serde_json::json!({
                                    "name": p.name, "steam_id": p.steam_id, "team_id": p.team_id,
                                    "squad_id": p.squad_id, "role": p.role, "kills": p.kills,
                                    "deaths": p.deaths, "score": p.score, "ping": p.ping, "is_admin": p.is_admin,
                                    "is_leader": p.is_leader,
                                })).collect::<Vec<_>>(),
                                "squads": squads.iter().map(|s| {
                                    let leader_from_player = leader_map.get(&s.squad_id);
                                    let leader_name = s.leader_name.clone()
                                        .or_else(|| leader_from_player.and_then(|l| l["name"].as_str().map(String::from)));
                                    let leader_steam_id = s.leader_steam_id.clone()
                                        .or_else(|| leader_from_player.and_then(|l| l["steam_id"].as_str().map(String::from)));
                                    serde_json::json!({
                                        "name": s.name, "creator": s.creator, "team_id": s.team_id,
                                        "squad_id": s.squad_id,
                                        "leader_name": leader_name,
                                        "leader_steam_id": leader_steam_id,
                                    })
                                }).collect::<Vec<_>>(),
                                "squad_leaders": squad_leaders,
                                "teams": teams,
                                "map_name": map_name, "game_mode": game_mode,
                                "server_name": server_name, "player_count": player_count,
                                "max_players": max_players, "next_map": next_map,
                            });
                            if let Ok(mut cache) = server_states.write() {
                                cache.insert(sid.to_string(), state);
                                tracing::debug!(server_id = %sid, "服务器状态已缓存");
                            } else {
                                tracing::error!(server_id = %sid, "服务器状态缓存写入失败（锁污染）");
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    let mut send_task = tokio::spawn(async move {
        while let Some(cmd) = cmd_rx.recv().await {
            let json = match serde_json::to_string(&cmd) {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("Agent 消息序列化失败: {}", e);
                    continue;
                }
            };
            if ws_sender
                .send(Message::Text(json.into()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    tokio::select! {
        _ = &mut recv_task => {}
        _ = &mut send_task => {}
    }

    agent_pool.agents.write().await.remove(&server_id);
    // 清理该 Agent 相关的所有待处理请求
    agent_pool.pending.write().await.clear();
    tracing::info!("Agent 已断开: {}", server_id);
    crate::services::system_log::agent_event(&db_pool, "agent_ws", &format!("Agent 已断开 server_id={}", server_id)).await;
}
