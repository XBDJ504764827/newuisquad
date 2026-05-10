use std::collections::HashMap;
use std::sync::Arc;
use sqlx::PgPool;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration, Instant};
use crate::models::server_log::LogEntry;
use crate::rcon_client::pool::RconPool;

/// 玩家误杀状态
#[derive(Debug, Clone)]
struct PlayerTkState {
    tk_count: u32,
    apology_deadline: Option<Instant>,
    apologized: bool,
    kicked: bool,
}

struct TkTracker {
    /// key: (server_id, player_steamid64) -> state
    players: HashMap<(i32, String), PlayerTkState>,
    /// 反向索引：(server_id, player_name) -> steam_id，用于道歉时回退匹配
    name_to_steam: HashMap<(i32, String), String>,
}

impl TkTracker {
    fn new() -> Self {
        Self { players: HashMap::new(), name_to_steam: HashMap::new() }
    }

    fn record_tk(&mut self, server_id: i32, steam_id: &str, attacker_name: &str, max_tk: u32) -> (u32, bool) {
        let key = (server_id, steam_id.to_string());
        let entry = self.players.entry(key.clone()).or_insert_with(|| PlayerTkState {
            tk_count: 0,
            apology_deadline: None,
            apologized: false,
            kicked: false,
        });

        if entry.kicked {
            return (entry.tk_count, false);
        }

        entry.tk_count += 1;
        let need_kick_timer = entry.tk_count >= max_tk && !entry.apologized;
        // 建立反向索引
        if !attacker_name.is_empty() {
            self.name_to_steam.insert((server_id, attacker_name.to_string()), steam_id.to_string());
        }
        (entry.tk_count, need_kick_timer)
    }

    fn mark_apologized(&mut self, server_id: i32, steam_id: &str) -> bool {
        let key = (server_id, steam_id.to_string());
        if let Some(entry) = self.players.get_mut(&key) {
            if entry.apology_deadline.is_some() && !entry.kicked {
                entry.apologized = true;
                entry.apology_deadline = None;
                return true;
            }
        }
        false
    }

    /// 解析 steam_id：先直接查找，再通过 name 回退
    fn resolve_steam_id(&self, server_id: i32, raw_id: &str, player_name: &str) -> Option<String> {
        // 1. 直接匹配
        let key = (server_id, raw_id.to_string());
        if self.players.contains_key(&key) {
            return Some(raw_id.to_string());
        }
        // 2. 通过 name 回退
        if !player_name.is_empty() {
            let name_key = (server_id, player_name.to_string());
            if let Some(steam) = self.name_to_steam.get(&name_key) {
                if self.players.contains_key(&(server_id, steam.clone())) {
                    return Some(steam.clone());
                }
            }
        }
        None
    }

    fn mark_kicked(&mut self, server_id: i32, steam_id: &str) {
        let key = (server_id, steam_id.to_string());
        if let Some(entry) = self.players.get_mut(&key) {
            entry.kicked = true;
            entry.apology_deadline = None;
        }
    }

    fn set_apology_deadline(&mut self, server_id: i32, steam_id: &str, deadline: Instant) {
        let key = (server_id, steam_id.to_string());
        if let Some(entry) = self.players.get_mut(&key) {
            entry.apology_deadline = Some(deadline);
        }
    }

    fn is_apologized(&self, server_id: i32, steam_id: &str) -> bool {
        let key = (server_id, steam_id.to_string());
        self.players.get(&key).map(|e| e.apologized).unwrap_or(false)
    }

    fn is_kicked(&self, server_id: i32, steam_id: &str) -> bool {
        let key = (server_id, steam_id.to_string());
        self.players.get(&key).map(|e| e.kicked).unwrap_or(false)
    }
}

/// 尝试从日志行解析 TK 事件。返回 (attacker_name, attacker_steamid64, victim_name)
fn parse_tk(line: &str) -> Option<(String, String, String)> {
    if line.to_lowercase().contains("teamkill") || line.to_lowercase().contains("team kill") {
        let mut attacker = String::new();
        let mut attacker_id = String::new();
        let mut victim = String::new();

        for pattern in &["Attacker=", "attacker=", "killer=", "Killer="] {
            if let Some(pos) = line.find(pattern) {
                let rest = &line[pos + pattern.len()..];
                if let Some(end) = rest.find(|c: char| c == ',' || c == ' ') {
                    attacker = rest[..end].trim_matches('"').to_string();
                }
                break;
            }
        }

        if let Some(start) = line.find('(') {
            let rest = &line[start + 1..];
            if let Some(end) = rest.find(')') {
                let id = &rest[..end];
                if id.chars().all(|c| c.is_ascii_digit()) && id.len() >= 10 {
                    attacker_id = id.to_string();
                }
            }
        }

        for pattern in &["Victim=", "victim="] {
            if let Some(pos) = line.find(pattern) {
                let rest = &line[pos + pattern.len()..];
                if let Some(end) = rest.find(|c: char| c == ',' || c == ' ') {
                    victim = rest[..end].trim_matches('"').to_string();
                }
                break;
            }
        }

        if !attacker.is_empty() && !attacker_id.is_empty() {
            return Some((attacker, attacker_id, victim));
        }
    }

    if line.to_lowercase().contains("(teamkill)") {
        let parts: Vec<&str> = line.split_whitespace().collect();
        for i in 0..parts.len() {
            if parts[i].starts_with('(') && parts[i].ends_with(')') {
                let id = &parts[i][1..parts[i].len()-1];
                if id.chars().all(|c| c.is_ascii_digit()) && id.len() >= 10 {
                    let name = if i > 0 { parts[i-1].trim_matches('"').to_string() } else { "Unknown".to_string() };
                    let victim_name = parts.iter()
                        .skip(i+1)
                        .find(|p| p.starts_with('"'))
                        .map(|p| p.trim_matches('"').to_string())
                        .unwrap_or_else(|| "Unknown".to_string());
                    return Some((name, id.to_string(), victim_name));
                }
            }
        }
    }

    None
}

/// 尝试从日志行解析聊天消息。返回 (player_name, steam_id, message)
fn parse_chat(line: &str) -> Option<(String, String, String)> {
    if !line.contains("[Chat]") && !line.contains("Chat") {
        return None;
    }

    if let Some(chat_pos) = line.find("[Chat]").or_else(|| line.find("Chat:")) {
        let rest = &line[chat_pos..];
        let content = if rest.starts_with("[Chat]") {
            &rest[6..]
        } else {
            &rest[5..]
        };
        let content = content.trim();

        if let Some(colon_pos) = content.find(": ") {
            let header = &content[..colon_pos];
            let message = content[colon_pos + 2..].trim();

            if let Some(paren_start) = header.rfind('(') {
                if let Some(paren_end) = header.rfind(')') {
                    let steam_id = &header[paren_start + 1..paren_end];
                    let player_name = header[..paren_start].trim().to_string();

                    if steam_id.chars().all(|c| c.is_ascii_digit()) && steam_id.len() >= 10 {
                        return Some((player_name, steam_id.to_string(), message.to_string()));
                    }
                }
            }
        }
    }

    None
}

/// 后台启动误杀检测任务
pub fn start_tk_monitor(
    pool: PgPool,
    mut log_rx: tokio::sync::broadcast::Receiver<LogEntry>,
    rcon_pool: RconPool,
) -> tokio::task::JoinHandle<()> {
    let tracker = Arc::new(RwLock::new(TkTracker::new()));

    tokio::spawn(async move {
        tracing::info!("误杀检测服务已启动");

        loop {
            match log_rx.recv().await {
                Ok(entry) => {
                    let raw = entry.raw_line.as_deref().unwrap_or(&entry.message);
                    let server_id = entry.server_id;
                    if server_id == 0 { continue; }

                    // 1. 检测 TK 事件
                    if let Some((attacker_name, attacker_id, victim_name)) = parse_tk(raw) {
                        // 查询 TK 设置（含道歉关键字和广播消息）
                        let tk_config = match sqlx::query_as::<_, (i32, String, Option<String>, Option<String>)>(
                            "SELECT tk.apology_time_minutes, tk.apology_keyword, tk.notification_message, tk.tk_broadcast_message FROM tk_settings tk WHERE tk.server_id = $1 AND tk.enabled = true"
                        ).bind(server_id).fetch_optional(&pool).await {
                            Ok(Some(c)) => c,
                            Ok(None) => continue,
                            Err(e) => { tracing::error!(%e, "查询TK设置失败"); continue; }
                        };

                        let (apology_minutes, apology_keyword, notif_msg, tk_broadcast_msg) = tk_config;
                        let max_tk = match sqlx::query_scalar::<_, i32>(
                            "SELECT max_team_kills FROM tk_settings WHERE server_id = $1"
                        ).bind(server_id).fetch_optional(&pool).await {
                            Ok(Some(v)) => v as u32,
                            _ => 3,
                        };

                        let mut t = tracker.write().await;
                        let (tk_count, need_timer) = t.record_tk(server_id, &attacker_id, &attacker_name, max_tk);

                        // 发送黄字广播: <玩家名称>误伤了<被击倒玩家名称>，输入<道歉关键字>道歉
                        let broadcast_msg = tk_broadcast_msg.unwrap_or_else(|| {
                            format!("{}误伤了{}，输入{}道歉", attacker_name, victim_name, apology_keyword)
                        });
                        send_rcon_broadcast(&rcon_pool, server_id, &broadcast_msg).await;

                        // 发送 AdminWarn 警告
                        let default_warn = format!("您误伤了友方{}，请在{}分钟内输入{}道歉，超过最大误杀数({}人)将被踢出", victim_name, apology_minutes, apology_keyword, max_tk);
                        let warn_msg = notif_msg.unwrap_or(default_warn)
                            .replace("{time}", &format!("{}分钟", apology_minutes));
                        match send_rcon_warn(&rcon_pool, server_id, &attacker_name, &warn_msg).await {
                            Ok(_) => tracing::info!(server_id, player = %attacker_name, tk_count, "已发送误杀警告"),
                            Err(e) => tracing::error!(server_id, player = %attacker_name, error = %e, "AdminWarn 发送失败"),
                        }

                        if need_timer {
                            let deadline = Instant::now() + Duration::from_secs((apology_minutes as u64) * 60);
                            t.set_apology_deadline(server_id, &attacker_id, deadline);

                            let tracker_clone = tracker.clone();
                            let attacker_id_copy = attacker_id.to_string();
                            let attacker_name_copy = attacker_name.to_string();
                            let rcon_pool_clone = rcon_pool.clone();

                            tokio::spawn(async move {
                                sleep(Duration::from_secs((apology_minutes as u64) * 60)).await;

                                let t = tracker_clone.read().await;
                                if t.is_apologized(server_id, &attacker_id_copy) || t.is_kicked(server_id, &attacker_id_copy) {
                                    return;
                                }
                                drop(t);

                                let kick_msg = format!("您因累计误杀队友超过{}次且未在{}分钟内道歉，已被踢出服务器", max_tk, apology_minutes);
                                match send_rcon_kick(&rcon_pool_clone, server_id, &attacker_name_copy, &kick_msg).await {
                                    Ok(_) => tracing::info!(server_id, player = %attacker_name_copy, "玩家因未道歉被踢出"),
                                    Err(e) => tracing::error!(server_id, player = %attacker_name_copy, error = %e, "AdminKick 发送失败"),
                                }

                                tracker_clone.write().await.mark_kicked(server_id, &attacker_id_copy);
                            });
                        }
                    }

                    // 2. 检测道歉消息（使用配置的道歉关键字，支持回退匹配）
                    if let Some((player_name, steam_id, message)) = parse_chat(raw) {
                        let upper = message.to_uppercase();
                        // 查询该服务器的道歉关键字
                        let apology_kw = match sqlx::query_scalar::<_, String>(
                            "SELECT apology_keyword FROM tk_settings WHERE server_id = $1 AND enabled = true"
                        ).bind(server_id).fetch_optional(&pool).await {
                            Ok(Some(kw)) => kw,
                            _ => "sry".to_string(),
                        };
                        let kw_upper = apology_kw.to_uppercase();
                        if upper.contains(&kw_upper) || upper == format!("!{}", apology_kw).to_uppercase() {
                            let mut t = tracker.write().await;
                            // 先尝试直接匹配
                            if t.mark_apologized(server_id, &steam_id) {
                                tracing::info!(server_id, %steam_id, %player_name, "玩家已道歉（直接匹配），取消踢出");
                                send_rcon_broadcast(&rcon_pool, server_id, "道歉成功").await;
                            } else if let Some(resolved) = t.resolve_steam_id(server_id, &steam_id, &player_name) {
                                // 回退匹配
                                if resolved != steam_id && t.mark_apologized(server_id, &resolved) {
                                    tracing::info!(server_id, %steam_id, resolved = %resolved, %player_name, "玩家已道歉（回退匹配），取消踢出");
                                    send_rcon_broadcast(&rcon_pool, server_id, "道歉成功").await;
                                }
                            }
                        }
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(skipped = n, "误杀检测滞后");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    tracing::info!("日志广播关闭，误杀检测停止");
                    break;
                }
            }
        }
    })
}

async fn send_rcon_warn(rcon_pool: &RconPool, server_id: i32, player_name: &str, message: &str) -> Result<(), String> {
    let cmd = format!("AdminWarn \"{}\" \"{}\"", player_name, message);
    rcon_pool.execute_by_server_id(server_id, &cmd).await?;
    Ok(())
}

async fn send_rcon_broadcast(rcon_pool: &RconPool, server_id: i32, message: &str) {
    let cmd = format!("AdminBroadcast \"{}\"", message);
    let _ = rcon_pool.execute_by_server_id(server_id, &cmd).await;
}

async fn send_rcon_kick(rcon_pool: &RconPool, server_id: i32, player_name: &str, reason: &str) -> Result<(), String> {
    let cmd = format!("AdminKick \"{}\" \"{}\"", player_name, reason);
    rcon_pool.execute_by_server_id(server_id, &cmd).await?;
    Ok(())
}
