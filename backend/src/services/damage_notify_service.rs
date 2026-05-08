use std::collections::HashMap;
use std::sync::Arc;
use sqlx::PgPool;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration, Instant};
use crate::models::damage_notify_settings::{DamageNotifySettings, UpdateDamageNotifyRequest};
use crate::models::server_log::LogEntry;
use crate::repositories::damage_notify_repo;
use crate::rcon_client::pool::RconPool;
use crate::services::system_log;

// ═══ API 层使用的 CRUD 函数 ═══

pub async fn get(pool: &PgPool, server_id: i32) -> Result<DamageNotifySettings, sqlx::Error> {
    damage_notify_repo::get_or_create(pool, server_id).await
}

pub async fn update(pool: &PgPool, server_id: i32, req: UpdateDamageNotifyRequest) -> Result<DamageNotifySettings, sqlx::Error> {
    damage_notify_repo::update(pool, server_id, &req).await
}

// ════════════════════════════════════════════
//  统一伤害与误伤通知服务
//  通过 server_states 缓存中的 team_id 判断敌友
// ════════════════════════════════════════════

/// 玩家误伤状态
#[derive(Debug, Clone)]
struct PlayerTkState {
    tk_count: u32,
    apology_deadline: Option<Instant>,
    apologized: bool,
    kicked: bool,
    timer_gen: u64, // 计时器代数，每次新误伤 +1，旧计时器检测到过期则跳过
}

struct TkTracker {
    players: HashMap<(i32, String), PlayerTkState>,
}

impl TkTracker {
    fn new() -> Self { Self { players: HashMap::new() } }

    fn record_tk(&mut self, server_id: i32, steam_id: &str) -> (u32, u64) {
        let key = (server_id, steam_id.to_string());
        let entry = self.players.entry(key.clone()).or_insert_with(|| PlayerTkState {
            tk_count: 0, apology_deadline: None, apologized: false, kicked: false, timer_gen: 0,
        });
        if entry.kicked { return (entry.tk_count, entry.timer_gen); }
        entry.tk_count += 1;
        entry.apology_deadline = None;
        entry.apologized = false;
        entry.timer_gen += 1;
        (entry.tk_count, entry.timer_gen)
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

    fn mark_kicked(&mut self, server_id: i32, steam_id: &str) {
        if let Some(entry) = self.players.get_mut(&(server_id, steam_id.to_string())) {
            entry.kicked = true;
            entry.apology_deadline = None;
        }
    }

    fn set_apology_deadline(&mut self, server_id: i32, steam_id: &str, deadline: Instant) {
        if let Some(entry) = self.players.get_mut(&(server_id, steam_id.to_string())) {
            entry.apology_deadline = Some(deadline);
        }
    }

    fn is_apologized(&self, server_id: i32, steam_id: &str) -> bool {
        self.players.get(&(server_id, steam_id.to_string())).map(|e| e.apologized).unwrap_or(false)
    }

    fn is_kicked(&self, server_id: i32, steam_id: &str) -> bool {
        self.players.get(&(server_id, steam_id.to_string())).map(|e| e.kicked).unwrap_or(false)
    }

    fn is_timer_current(&self, server_id: i32, steam_id: &str, gen: u64) -> bool {
        self.players.get(&(server_id, steam_id.to_string()))
            .map(|e| e.timer_gen == gen).unwrap_or(false)
    }
}

/// 从 server_states 缓存中查找玩家的 PlayerID（通过名称）
fn find_player_id(
    server_states: &HashMap<String, serde_json::Value>,
    server_id: i32,
    player_name: &str,
) -> Option<i32> {
    let state = server_states.get(&server_id.to_string())?;
    let players = state.get("players")?.as_array()?;
    for p in players {
        if p.get("name")?.as_str()? == player_name {
            return p.get("player_id")?.as_i64().map(|id| id as i32);
        }
    }
    None
}

/// 通过 steam_id 从缓存查找 PlayerID
fn find_player_id_by_steam(
    server_states: &HashMap<String, serde_json::Value>,
    server_id: i32,
    steam_id: &str,
) -> Option<i32> {
    let state = server_states.get(&server_id.to_string())?;
    let players = state.get("players")?.as_array()?;
    for p in players {
        if p.get("steam_id")?.as_str()? == steam_id {
            return p.get("player_id")?.as_i64().map(|id| id as i32);
        }
    }
    None
}

/// 通过 server_states 缓存判断两名玩家是否同队
/// 返回 Some(true) = 同队（友军），Some(false) = 不同队（敌方），None = 无法判断
fn is_same_team(
    server_states: &HashMap<String, serde_json::Value>,
    server_id: i32,
    attacker_steam64: &str,
    attacker_name: &str,
    victim_name: &str,
) -> Option<bool> {
    let state = server_states.get(&server_id.to_string())?;
    let players = state.get("players")?.as_array()?;

    // 查找攻击者 team_id：优先用 steam_id 匹配，回退到名称匹配
    let attacker_team = players.iter().find(|p| {
        if !attacker_steam64.is_empty() {
            p.get("steam_id").and_then(|s| s.as_str()) == Some(attacker_steam64)
        } else {
            p.get("name").and_then(|s| s.as_str()) == Some(attacker_name)
        }
    })?.get("team_id")?.as_i64()?;

    // 查找受害者 team_id
    let victim_team = players.iter().find(|p| {
        p.get("name").and_then(|s| s.as_str()) == Some(victim_name)
    })?.get("team_id")?.as_i64()?;

    Some(attacker_team == victim_team)
}

/// 从日志行解析聊天消息
fn parse_chat(line: &str) -> Option<(String, String, String)> {
    if !line.contains("[Chat]") && !line.contains("Chat") { return None; }
    if let Some(chat_pos) = line.find("[Chat]").or_else(|| line.find("Chat:")) {
        let rest = &line[chat_pos..];
        let content = if rest.starts_with("[Chat]") { &rest[6..] } else { &rest[5..] };
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

pub fn start_damage_notify(
    pool: PgPool,
    mut log_rx: tokio::sync::broadcast::Receiver<LogEntry>,
    server_states: Arc<tokio::sync::RwLock<HashMap<String, serde_json::Value>>>,
    rcon_pool: RconPool,
) -> tokio::task::JoinHandle<()> {
    let tracker = Arc::new(RwLock::new(TkTracker::new()));

    let pool_clone = pool.clone();
    tokio::spawn(async move {
        tracing::info!("统一伤害与误伤通知服务已启动");
        system_log::backend_info(&pool_clone, "damage_notify", "伤害与误伤通知服务已启动").await;

        loop {
            match log_rx.recv().await {
                Ok(entry) => {
                    let raw = entry.raw_line.as_deref().unwrap_or(&entry.message);
                    let server_id = entry.server_id;
                    if server_id == 0 || raw.is_empty() { continue; }

                    if let Some(event) = crate::services::squad_log_parser::parse_line(raw) {
                        use crate::services::squad_log_parser::ParsedEvent;
                        if let ParsedEvent::KillEvent {
                            ref attacker_name, ref attacker_steam64,
                            ref victim_name, damage, ..
                        } = event {
                            let attacker = attacker_name.as_str();
                            let victim = victim_name.as_str();
                            if attacker.is_empty() || victim.is_empty() { continue; }

                            // 1. 全局开关检查
                            let global_enabled = match sqlx::query_as::<_, (bool,)>(
                                "SELECT enabled FROM damage_notify_settings WHERE server_id = $1"
                            ).bind(server_id).fetch_optional(&pool).await {
                                Ok(Some((enabled,))) => enabled,
                                _ => false,
                            };
                            if !global_enabled { continue; }

                            // 2. 通过 server_states 缓存中的 team_id 判断敌友
                            let states = server_states.read().await;
                            let same_team = is_same_team(&states, server_id, attacker_steam64, attacker, victim);
                            let attacker_pid = find_player_id(&states, server_id, attacker)
                                .or_else(|| find_player_id_by_steam(&states, server_id, attacker_steam64));
                            drop(states);

                            match same_team {
                                Some(true) => {
                                    // 友方误伤
                                    handle_teamkill(
                                        &pool, &tracker, &server_states, server_id,
                                        attacker, attacker_steam64, victim, damage,
                                        attacker_pid, &rcon_pool,
                                    ).await;
                                }
                                Some(false) => {
                                    // 敌方伤害
                                    handle_enemy_damage(
                                        &pool, server_id,
                                        attacker, victim, damage,
                                        attacker_pid, &rcon_pool,
                                    ).await;
                                }
                                None => {
                                    // 无法判断队伍（玩家不在缓存中），跳过
                                    tracing::debug!(server_id, attacker = %attacker, victim = %victim, "无法从缓存判断敌友，跳过伤害通知");
                                }
                            }
                        }
                    }

                    // 检测道歉消息
                    if let Some((_player_name, steam_id, message)) = parse_chat(raw) {
                        process_apology(&pool, &tracker, server_id, &steam_id, &message, &rcon_pool).await;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(skipped = n, "伤害通知服务滞后");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
        tracing::info!("统一伤害与误伤通知服务已停止");
        system_log::backend_info(&pool_clone, "damage_notify", "伤害与误伤通知服务已停止").await;
    })
}

/// 处理敌方伤害通知
async fn handle_enemy_damage(
    pool: &PgPool, server_id: i32,
    attacker: &str, victim: &str, damage: f64,
    attacker_pid: Option<i32>,
    rcon_pool: &RconPool,
) {
    // 检查敌方伤害通知子开关
    let notify_damage = match sqlx::query_as::<_, (bool,)>(
        "SELECT notify_damage FROM damage_notify_settings WHERE server_id = $1"
    ).bind(server_id).fetch_optional(pool).await {
        Ok(Some((nd,))) => nd,
        _ => false,
    };
    if !notify_damage { return; }

    let pid_str = attacker_pid.map(|id| id.to_string()).unwrap_or_else(|| attacker.to_string());
    let cmd = format!("AdminWarn {} 你对{}造成了{:.0}点伤害", pid_str, victim, damage);
    send_rcon_cmd(pool, server_id, &cmd, rcon_pool).await;
}

/// 处理友方误伤 — 每次伤害立即广播并启动道歉倒计时
async fn handle_teamkill(
    pool: &PgPool,
    tracker: &Arc<RwLock<TkTracker>>,
    server_states: &Arc<tokio::sync::RwLock<HashMap<String, serde_json::Value>>>,
    server_id: i32,
    attacker: &str,
    attacker_steam64: &str,
    victim: &str,
    damage: f64,
    attacker_pid: Option<i32>,
    rcon_pool: &RconPool,
) {
    // 查询 TK 设置（误伤子开关）
    let tk_config = match sqlx::query_as::<_, (bool, i32, String)>(
        "SELECT enabled, apology_time_minutes, apology_keyword \
         FROM tk_settings WHERE server_id = $1"
    ).bind(server_id).fetch_optional(pool).await {
        Ok(Some(c)) => c,
        _ => return,
    };
    let (tk_enabled, apology_minutes, apology_keyword) = tk_config;
    if !tk_enabled { return; }

    let pid_str = attacker_pid.map(|id| id.to_string()).unwrap_or_else(|| attacker.to_string());

    // 1. AdminBroadcast 黄字广播（按用户要求格式）
    let broadcast_msg = format!(
        "{}误伤了队友{}，请输入{}道歉否则将在{}分钟后被踢出",
        attacker, victim, apology_keyword, apology_minutes
    );
    send_rcon_cmd(pool, server_id, &format!("AdminBroadcast {}", broadcast_msg), rcon_pool).await;

    // 2. 更新 TK 计数并重置道歉状态，获取当前计时器代数
    let (tk_count, timer_gen) = {
        let mut t = tracker.write().await;
        t.record_tk(server_id, attacker_steam64)
    };
    tracing::info!(server_id, player = %attacker, tk_count, victim = %victim, damage, timer_gen, "误伤事件");

    // 3. 启动/重置道歉倒计时（每次误伤都重新计时）
    let deadline = Instant::now() + Duration::from_secs((apology_minutes as u64) * 60);
    tracker.write().await.set_apology_deadline(server_id, attacker_steam64, deadline);

    let tracker_clone = tracker.clone();
    let pool_clone = pool.clone();
    let server_states_clone = server_states.clone();
    let attacker_id = attacker_steam64.to_string();
    let attacker_name = attacker.to_string();
    let apology_kw = apology_keyword.clone();
    let rcon_pool_clone = rcon_pool.clone();

    tokio::spawn(async move {
        sleep(Duration::from_secs((apology_minutes as u64) * 60)).await;

        let t = tracker_clone.read().await;
        // 已道歉、已踢出、或有新误伤重置了计时器 → 跳过
        if t.is_apologized(server_id, &attacker_id)
            || t.is_kicked(server_id, &attacker_id)
            || !t.is_timer_current(server_id, &attacker_id, timer_gen)
        {
            return;
        }
        drop(t);

        // 从缓存中获取最新 PlayerID（可能在倒计时期间变化）
        let kick_pid = {
            let states = server_states_clone.read().await;
            find_player_id_by_steam(&states, server_id, &attacker_id)
                .or_else(|| find_player_id(&states, server_id, &attacker_name))
        };

        let kick_reason = format!(
            "您因误伤队友后未在{}分钟内输入{}道歉，已被踢出服务器",
            apology_minutes, apology_kw
        );

        if let Some(pid) = kick_pid {
            let kick_cmd = format!("AdminKickById {} {}", pid, kick_reason);
            send_rcon_cmd(&pool_clone, server_id, &kick_cmd, &rcon_pool_clone).await;
            tracing::info!(server_id, player = %attacker_name, player_id = pid, "玩家因未道歉被踢出 (AdminKickById)");
        } else {
            // 回退：无法获取 PlayerID，尝试用名称踢出
            let kick_cmd = format!("AdminKick {} {}", attacker_name, kick_reason);
            send_rcon_cmd(&pool_clone, server_id, &kick_cmd, &rcon_pool_clone).await;
            tracing::warn!(server_id, player = %attacker_name, "无法获取 PlayerID，使用 AdminKick 名称踢出");
        }

        tracker_clone.write().await.mark_kicked(server_id, &attacker_id);
    });
}

/// 处理道歉消息
async fn process_apology(
    pool: &PgPool,
    tracker: &Arc<RwLock<TkTracker>>,
    server_id: i32,
    steam_id: &str,
    message: &str,
    rcon_pool: &RconPool,
) {
    let apology_kw = match sqlx::query_as::<_, (String,)>(
        "SELECT apology_keyword FROM tk_settings WHERE server_id = $1 AND enabled = true"
    ).bind(server_id).fetch_optional(pool).await {
        Ok(Some((kw,))) => kw,
        _ => return,
    };

    let upper = message.to_uppercase();
    let kw_upper = apology_kw.to_uppercase();
    if !upper.contains(&kw_upper) {
        return;
    }

    let mut t = tracker.write().await;
    if t.mark_apologized(server_id, steam_id) {
        tracing::info!(server_id, %steam_id, "玩家已道歉，取消踢出");
        send_rcon_cmd(pool, server_id, "AdminBroadcast 道歉成功，已取消踢出", rcon_pool).await;
    }
}

// ═══ RCON 辅助函数（通过连接池复用连接） ═══

async fn send_rcon_cmd(pool: &PgPool, server_id: i32, cmd: &str, rcon_pool: &RconPool) {
    let creds = match sqlx::query_as::<_, (String, i32, String)>(
        "SELECT ip, rcon_port, rcon_password FROM servers WHERE id = $1"
    ).bind(server_id).fetch_optional(pool).await {
        Ok(Some(c)) => c,
        _ => return,
    };
    let (ip, port, password) = creds;
    if password.is_empty() { return; }
    match rcon_pool.execute(&ip, port as u16, &password, cmd).await {
        Ok(_) => {}
        Err(e) => tracing::warn!(%ip, %port, %e, "RCON 命令执行失败"),
    }
}
