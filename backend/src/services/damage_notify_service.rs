use std::collections::HashMap;
use std::sync::Arc;
use sqlx::PgPool;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration, Instant};
use crate::models::damage_notify_settings::{DamageNotifySettings, UpdateDamageNotifyRequest};
use crate::models::server_log::LogEntry;
use crate::repositories::damage_notify_repo;
use crate::rcon_client::squad::SquadRcon;
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
//  合并原 damage_notify_service + tk_service
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

/// 从 server_states 缓存中查找玩家的 PlayerID
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
                            ref victim_name, damage, is_teamkill, ..
                        } = event {
                            let attacker = attacker_name.as_str();
                            let victim = victim_name.as_str();
                            if attacker.is_empty() || victim.is_empty() { continue; }

                            // 读取服务器状态缓存获取 PlayerID
                            let states = server_states.read().await;
                            let attacker_pid = find_player_id(&states, server_id, attacker)
                                .or_else(|| find_player_id_by_steam(&states, server_id, attacker_steam64));

                            if is_teamkill {
                                handle_teamkill(
                                    &pool, &tracker, server_id,
                                    attacker, attacker_steam64, victim, damage,
                                    attacker_pid,
                                ).await;
                            } else {
                                handle_enemy_damage(
                                    &pool, server_id,
                                    attacker, victim, damage,
                                    attacker_pid,
                                ).await;
                            }
                        }
                    }

                    // 检测道歉消息
                    if let Some((_player_name, steam_id, message)) = parse_chat(raw) {
                        process_apology(&pool, &tracker, server_id, &steam_id, &message).await;
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
) {
    let settings = match sqlx::query_as::<_, (bool, bool)>(
        "SELECT enabled, notify_damage FROM damage_notify_settings WHERE server_id = $1"
    ).bind(server_id).fetch_optional(pool).await {
        Ok(Some(s)) => s,
        _ => return,
    };
    let (enabled, notify_damage) = settings;
    if !enabled || !notify_damage { return; }

    let pid_str = attacker_pid.map(|id| id.to_string()).unwrap_or_else(|| attacker.to_string());
    let cmd = format!("AdminWarn {} 你对{}造成了{:.0}点伤害", pid_str, victim, damage);
    send_rcon_cmd(pool, server_id, &cmd).await;
}

/// 处理友方误伤 — 每次伤害立即广播并启动道歉倒计时
async fn handle_teamkill(
    pool: &PgPool,
    tracker: &Arc<RwLock<TkTracker>>,
    server_id: i32,
    attacker: &str,
    attacker_steam64: &str,
    victim: &str,
    damage: f64,
    attacker_pid: Option<i32>,
) {
    // 查询 TK 设置
    let tk_config = match sqlx::query_as::<_, (String, i32, String, i32, String, Option<String>)>(
        "SELECT s.ip, s.rcon_port, s.rcon_password, tk.apology_time_minutes, tk.apology_keyword, tk.tk_broadcast_message \
         FROM servers s JOIN tk_settings tk ON s.id = tk.server_id \
         WHERE s.id = $1 AND tk.enabled = true"
    ).bind(server_id).fetch_optional(pool).await {
        Ok(Some(c)) => c,
        _ => return,
    };
    let (ip, rcon_port, rcon_password, apology_minutes, apology_keyword, tk_broadcast_msg) = tk_config;

    let pid_str = attacker_pid.map(|id| id.to_string()).unwrap_or_else(|| attacker.to_string());

    // 1. AdminWarn 通知攻击者
    let warn_cmd = format!("AdminWarn {} 你对队友{}造成了{:.0}点伤害", pid_str, victim, damage);
    send_rcon_cmd_direct(&ip, rcon_port as u16, &rcon_password, &warn_cmd).await;

    // 2. AdminBroadcast 黄字广播（含道歉关键字和道歉时间）
    let broadcast_msg = tk_broadcast_msg.unwrap_or_else(|| {
        format!("{}误伤了队友{}，输入{}道歉，否则将在{}分钟后被踢出",
            attacker, victim, apology_keyword, apology_minutes)
    });
    send_rcon_broadcast(&ip, rcon_port as u16, &rcon_password, &broadcast_msg).await;

    // 3. 更新 TK 计数并重置道歉状态，获取当前计时器代数
    let (tk_count, timer_gen) = {
        let mut t = tracker.write().await;
        t.record_tk(server_id, attacker_steam64)
    };
    tracing::info!(server_id, player = %attacker, tk_count, victim = %victim, damage, timer_gen, "误伤事件");

    // 4. 启动/重置道歉倒计时（每次误伤都重新计时）
    let deadline = Instant::now() + Duration::from_secs((apology_minutes as u64) * 60);
    tracker.write().await.set_apology_deadline(server_id, attacker_steam64, deadline);

    let tracker_clone = tracker.clone();
    let attacker_id = attacker_steam64.to_string();
    let attacker_name = attacker.to_string();
    let ip_copy = ip.clone();
    let pass_copy = rcon_password.clone();

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

        let kick_cmd = format!("AdminKick {} 您因误伤队友后未在{}分钟内输入{}道歉，已被踢出服务器",
            &attacker_name, apology_minutes, apology_keyword);

        if let Ok(mut rcon) = SquadRcon::connect(&ip_copy, rcon_port as u16, &pass_copy).await {
            let _ = rcon.execute(&kick_cmd).await;
            tracing::info!(server_id, player = %attacker_name, "玩家因未道歉被踢出");
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
) {
    let (apology_kw, ip, rcon_port, rcon_password) = match sqlx::query_as::<_, (String, String, i32, String)>(
        "SELECT tk.apology_keyword, s.ip, s.rcon_port, s.rcon_password \
         FROM tk_settings tk JOIN servers s ON s.id = tk.server_id \
         WHERE tk.server_id = $1 AND tk.enabled = true"
    ).bind(server_id).fetch_optional(pool).await {
        Ok(Some(c)) => c,
        _ => return,
    };

    let upper = message.to_uppercase();
    let kw_upper = apology_kw.to_uppercase();
    if !upper.contains(&kw_upper) && upper != format!("!{}", apology_kw).to_uppercase() {
        return;
    }

    let mut t = tracker.write().await;
    if t.mark_apologized(server_id, steam_id) {
        tracing::info!(server_id, %steam_id, "玩家已道歉，取消踢出");
        send_rcon_broadcast(&ip, rcon_port as u16, &rcon_password, "道歉成功").await;
    }
}

// ═══ RCON 辅助函数 ═══

async fn send_rcon_cmd(pool: &PgPool, server_id: i32, cmd: &str) {
    let creds = match sqlx::query_as::<_, (String, i32, String)>(
        "SELECT ip, rcon_port, rcon_password FROM servers WHERE id = $1"
    ).bind(server_id).fetch_optional(pool).await {
        Ok(Some(c)) => c,
        _ => return,
    };
    send_rcon_cmd_direct(&creds.0, creds.1 as u16, &creds.2, cmd).await;
}

async fn send_rcon_cmd_direct(ip: &str, port: u16, password: &str, cmd: &str) {
    if password.is_empty() { return; }
    match SquadRcon::connect(ip, port, password).await {
        Ok(mut rcon) => { let _ = rcon.execute(cmd).await; }
        Err(e) => tracing::warn!(%ip, %port, %e, "RCON 连接失败"),
    }
}

async fn send_rcon_broadcast(ip: &str, port: u16, password: &str, message: &str) {
    let cmd = format!("AdminBroadcast \"{}\"", message);
    send_rcon_cmd_direct(ip, port, password, &cmd).await;
}
