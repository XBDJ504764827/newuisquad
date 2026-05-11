use std::collections::HashMap;
use std::sync::Arc;
use sqlx::PgPool;
use tokio::sync::RwLock;
use crate::models::server_log::LogEntry;
use crate::rcon_client::pool::RconPool;
use crate::services::system_log;

/// 每个玩家在每台服务器上的违规建队计数
/// key: (server_id, steam_id)
type AttemptTracker = Arc<RwLock<HashMap<(i32, String), u32>>>;

/// 队伍设置服务：监听小队创建事件，执行建队广播和队长时长检测
pub fn start_team_service(
    pool: PgPool,
    mut log_rx: tokio::sync::broadcast::Receiver<LogEntry>,
    server_states: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    rcon_pool: RconPool,
    steam_api_key: String,
) -> tokio::task::JoinHandle<()> {
    let attempts: AttemptTracker = Arc::new(RwLock::new(HashMap::new()));

    tokio::spawn(async move {
        tracing::info!("队伍设置服务已启动");
        system_log::backend_info(&pool, "team_service", "队伍设置服务已启动").await;

        loop {
            match log_rx.recv().await {
                Ok(entry) => {
                    let raw = entry.raw_line.as_deref().unwrap_or(&entry.message);
                    let server_id = entry.server_id;
                    if server_id == 0 || raw.is_empty() { continue; }

                    // 解析小队创建事件
                    if let Some(event) = crate::services::squad_log_parser::parse_line(raw) {
                        use crate::services::squad_log_parser::ParsedEvent;
                        if let ParsedEvent::SquadCreation { ref player_name, ref steam64, ref squad_id, ref squad_name, .. } = event {
                            if player_name.is_empty() { continue; }

                            // 查询 team_settings 配置
                            let config = match sqlx::query_as::<_, (bool, bool, i32, i32, i32)>(
                                "SELECT create_team_broadcast, captain_time_check, captain_min_playtime, \
                                 captain_check_min_players, max_create_team_attempts \
                                 FROM team_settings WHERE server_id = $1"
                            ).bind(server_id).fetch_optional(&pool).await {
                                Ok(Some(c)) => c,
                                Ok(None) => continue,
                                Err(e) => { tracing::error!(server_id, error = %e, "查询队伍设置失败"); continue; }
                            };

                            let (create_broadcast, captain_check, min_playtime, min_players, max_attempts) = config;

                            // 1. 建队广播
                            if create_broadcast {
                                let msg = format!("AdminBroadcast {}创建了小队", player_name);
                                send_rcon_cmd(&rcon_pool, server_id, &msg).await;
                                tracing::info!(server_id, player = %player_name, squad = %squad_name, "建队广播已发送");
                            }

                            // 2. 队长游戏时长检测
                            if captain_check && !steam64.is_empty() {
                                // 检查在线人数是否达到时长检查生效人数
                                let player_count = get_online_player_count(&server_states, server_id).await;
                                if player_count < min_players as usize {
                                    tracing::debug!(server_id, player_count, min_players, "在线人数未达到队长时长检测阈值，跳过");
                                    continue;
                                }

                                // 在 Steam API 调用前先从缓存获取 team_id（避免 API 返回后缓存过期）
                                let team_id = find_player_team(&server_states, server_id, steam64).await;

                                // 通过 Steam API 查询玩家 Squad 游戏时长（分钟）
                                let playtime_minutes = fetch_squad_playtime(&steam_api_key, steam64).await;
                                tracing::info!(server_id, player = %player_name, %steam64, playtime_minutes, min_playtime, "队长时长检测");

                                if playtime_minutes < min_playtime as u64 {
                                    // 解散小队：squad_id 直接从日志事件获取
                                    if let Some(tid) = team_id {
                                        let disband_cmd = format!("AdminDisbandSquad {} {}", tid, squad_id);
                                        send_rcon_cmd(&rcon_pool, server_id, &disband_cmd).await;
                                        tracing::info!(server_id, player = %player_name, team_id = tid, %squad_id, "因时长不足解散小队");
                                    } else {
                                        // 无法确定 team_id，尝试两个队伍都解散
                                        for tid in [1, 2] {
                                            let disband_cmd = format!("AdminDisbandSquad {} {}", tid, squad_id);
                                            send_rcon_cmd(&rcon_pool, server_id, &disband_cmd).await;
                                        }
                                        tracing::info!(server_id, player = %player_name, %squad_id, "因时长不足解散小队（回退双队伍）");
                                    }

                                    // 立即警告玩家为何被解散
                                    let disband_warn = format!(
                                        "您的游戏时长不足，小队已被自动解散"
                                    );
                                    let warn_cmd = format!("AdminWarn \"{}\" \"{}\"", steam64, disband_warn);
                                    send_rcon_cmd(&rcon_pool, server_id, &warn_cmd).await;

                                    // 累计违规建队次数
                                    let key = (server_id, steam64.clone());
                                    let current_attempts = {
                                        let mut map = attempts.write().await;
                                        let count = map.entry(key).or_insert(0);
                                        *count += 1;
                                        *count
                                    };

                                    let remaining = max_attempts as u32 - current_attempts.min(max_attempts as u32);
                                    tracing::info!(server_id, player = %player_name, current_attempts, max_attempts, "违规建队计数");

                                    if current_attempts >= max_attempts as u32 {
                                        // 超过最大建队次数，踢出玩家
                                        let kick_reason = format!(
                                            "您的游戏时长不足且已超过最大建队次数({}次)，已被踢出服务器",
                                            max_attempts
                                        );
                                        if let Some(pid) = find_player_id(&server_states, server_id, steam64, player_name).await {
                                            let kick_cmd = format!("AdminKickById {} {}", pid, kick_reason);
                                            send_rcon_cmd(&rcon_pool, server_id, &kick_cmd).await;
                                        } else {
                                            let kick_cmd = format!("AdminKick \"{}\" \"{}\"", player_name, kick_reason);
                                            send_rcon_cmd(&rcon_pool, server_id, &kick_cmd).await;
                                        }
                                        tracing::info!(server_id, player = %player_name, "因超过最大建队次数被踢出");
                                        // 踢出后重置计数
                                        let mut map = attempts.write().await;
                                        map.remove(&(server_id, steam64.clone()));
                                    } else {
                                        // 警告玩家（告知剩余次数）
                                        let warn_msg = format!(
                                            "您的游戏时长无法建立小队，再违规{}次将被踢出服务器",
                                            remaining
                                        );
                                        if let Some(pid) = find_player_id(&server_states, server_id, steam64, player_name).await {
                                            let warn_cmd = format!("AdminWarn {} \"{}\"", pid, warn_msg);
                                            send_rcon_cmd(&rcon_pool, server_id, &warn_cmd).await;
                                        } else {
                                            let warn_cmd = format!("AdminWarn \"{}\" \"{}\"", player_name, warn_msg);
                                            send_rcon_cmd(&rcon_pool, server_id, &warn_cmd).await;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(skipped = n, "队伍设置服务滞后");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
        tracing::info!("队伍设置服务已停止");
        system_log::backend_info(&pool, "team_service", "队伍设置服务已停止").await;
    })
}

/// 从 server_states 缓存获取在线玩家数
async fn get_online_player_count(
    server_states: &Arc<RwLock<HashMap<String, serde_json::Value>>>,
    server_id: i32,
) -> usize {
    let states = server_states.read().await;
    states.get(&server_id.to_string())
        .and_then(|s| s.get("players"))
        .and_then(|p| p.as_array())
        .map(|a| a.len())
        .unwrap_or(0)
}

/// 从 server_states 缓存通过 steam_id 查找玩家的 PlayerID
async fn find_player_id(
    server_states: &Arc<RwLock<HashMap<String, serde_json::Value>>>,
    server_id: i32,
    steam_id: &str,
    player_name: &str,
) -> Option<i64> {
    let states = server_states.read().await;
    let state = states.get(&server_id.to_string())?;
    let players = state.get("players")?.as_array()?;
    // 优先用 steam_id 匹配
    for p in players {
        if p.get("steam_id").and_then(|s| s.as_str()) == Some(steam_id) {
            return p.get("player_id").and_then(|id| id.as_i64());
        }
    }
    // 回退用名字匹配
    for p in players {
        if p.get("name").and_then(|n| n.as_str()) == Some(player_name) {
            return p.get("player_id").and_then(|id| id.as_i64());
        }
    }
    None
}

/// 从 server_states 缓存查找玩家所属的 team_id
async fn find_player_team(
    server_states: &Arc<RwLock<HashMap<String, serde_json::Value>>>,
    server_id: i32,
    steam_id: &str,
) -> Option<i64> {
    let states = server_states.read().await;
    let state = states.get(&server_id.to_string())?;
    let players = state.get("players")?.as_array()?;
    for p in players {
        if p.get("steam_id").and_then(|s| s.as_str()) == Some(steam_id) {
            return p.get("team_id").and_then(|t| t.as_i64());
        }
    }
    None
}

/// 通过 Steam API 查询玩家 Squad 游戏时长（返回分钟数）
/// Squad 的 App ID 为 393380
async fn fetch_squad_playtime(api_key: &str, steam_id: &str) -> u64 {
    if api_key.is_empty() || steam_id.is_empty() {
        return 0;
    }

    let url = format!(
        "https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/?key={}&steamid={}&include_played_free_games=true&appids_filter[0]=393380&format=json",
        api_key, steam_id
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default();

    match client.get(&url).send().await {
        Ok(resp) => {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                // response.games[0].playtime_forever (分钟)
                if let Some(games) = json["response"]["games"].as_array() {
                    for game in games {
                        if game["appid"].as_u64() == Some(393380) {
                            return game["playtime_forever"].as_u64().unwrap_or(0);
                        }
                    }
                }
            }
        }
        Err(e) => {
            tracing::warn!("Steam API 查询游戏时长失败: {}", e);
        }
    }

    0
}

async fn send_rcon_cmd(rcon_pool: &RconPool, server_id: i32, cmd: &str) {
    match rcon_pool.execute_by_server_id(server_id, cmd).await {
        Ok(_) => {}
        Err(e) => tracing::warn!(server_id, %e, "RCON 命令执行失败"),
    }
}
