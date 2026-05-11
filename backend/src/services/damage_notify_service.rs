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

// ═══ 模板渲染 ═══

/// 将模板中的 {{key}} 替换为 vars 中对应的值。
/// 未匹配的变量替换为 "未知"，模板为空则返回空字符串。
fn render_template(template: &str, vars: &[(&str, &str)]) -> String {
    let template = template.trim();
    if template.is_empty() {
        return String::new();
    }
    let mut result = template.to_string();
    for (key, value) in vars {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }
    // 替换剩余未匹配的 {{xxx}} 为 "未知"
    while let Some(start) = result.find("{{") {
        if let Some(end) = result[start..].find("}}") {
            result.replace_range(start..start + end + 2, "未知");
        } else {
            break;
        }
    }
    result
}

// ═══ 统一消息分发 ═══

/// 根据 mode 选择发送方式：
/// - "broadcast": AdminBroadcast 全服广播
/// - "warning_all": 给所有在线玩家发 AdminWarn
/// - "warning_related": 仅给 attacker + victim 发 AdminWarn（默认）
async fn dispatch_message(
    pool: &PgPool,
    server_id: i32,
    message: &str,
    mode: &str,
    attacker_steam64: &str,
    victim_name: &str,
    server_states: &Arc<tokio::sync::RwLock<HashMap<String, serde_json::Value>>>,
    rcon_pool: &RconPool,
) {
    if message.is_empty() { return; }

    match mode {
        "broadcast" => {
            let cmd = format!("AdminBroadcast {}", message);
            send_rcon_cmd(pool, server_id, &cmd, rcon_pool).await;
        }
        "warning_all" => {
            let states = server_states.read().await;
            if let Some(state) = states.get(&server_id.to_string()) {
                if let Some(players) = state.get("players").and_then(|p| p.as_array()) {
                    for p in players {
                        if let Some(pid) = p.get("player_id").and_then(|id| id.as_i64()) {
                            let cmd = format!("AdminWarn {} {}", pid, message);
                            send_rcon_cmd(pool, server_id, &cmd, rcon_pool).await;
                        }
                    }
                }
            }
        }
        _ => {
            // warning_related: 仅发给攻击者和受害者
            let states = server_states.read().await;
            let attacker_pid = find_player_id_by_steam(&states, server_id, attacker_steam64);
            let victim_pid = find_player_id(&states, server_id, victim_name);
            drop(states);

            if let Some(pid) = attacker_pid {
                let cmd = format!("AdminWarn {} {}", pid, message);
                send_rcon_cmd(pool, server_id, &cmd, rcon_pool).await;
            }
            if let Some(pid) = victim_pid {
                let cmd = format!("AdminWarn {} {}", pid, message);
                send_rcon_cmd(pool, server_id, &cmd, rcon_pool).await;
            }
        }
    }
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
    last_apology_at: Option<Instant>, // 道歉预窗口：上次道歉成功的时间
}

struct TkTracker {
    players: HashMap<(i32, String), PlayerTkState>,
    /// 反向索引：(server_id, player_name) -> steam_id，用于道歉时通过玩家名回退匹配
    name_to_steam: HashMap<(i32, String), String>,
    /// 反向索引：(server_id, eos_id) -> steam_id，用于聊天ID不一致时回退匹配
    alt_id_to_steam: HashMap<(i32, String), String>,
}

impl TkTracker {
    fn new() -> Self { Self { players: HashMap::new(), name_to_steam: HashMap::new(), alt_id_to_steam: HashMap::new() } }

    /// 记录 TK 并建立反向索引（玩家名 -> steam_id）
    fn record_tk(&mut self, server_id: i32, steam_id: &str, attacker_name: &str) -> (u32, u64) {
        let key = (server_id, steam_id.to_string());
        let entry = self.players.entry(key.clone()).or_insert_with(|| PlayerTkState {
            tk_count: 0, apology_deadline: None, apologized: false, kicked: false, timer_gen: 0,
            last_apology_at: None,
        });
        if entry.kicked { return (entry.tk_count, entry.timer_gen); }
        entry.tk_count += 1;
        entry.apology_deadline = None;
        entry.apologized = false;
        entry.timer_gen += 1;
        // 建立反向索引：name -> steam_id
        if !attacker_name.is_empty() {
            self.name_to_steam.insert((server_id, attacker_name.to_string()), steam_id.to_string());
        }
        (entry.tk_count, entry.timer_gen)
    }

    /// 注册备用 ID（如 EOS ID）到 steam_id 的映射
    fn register_alt_id(&mut self, server_id: i32, alt_id: &str, steam_id: &str) {
        if !alt_id.is_empty() && !steam_id.is_empty() {
            self.alt_id_to_steam.insert((server_id, alt_id.to_string()), steam_id.to_string());
        }
    }

    /// 解析 steam_id：先直接查找，再通过 name 和 alt_id 回退
    fn resolve_steam_id(&self, server_id: i32, raw_id: &str, player_name: &str) -> Option<String> {
        // 1. 直接匹配
        let key = (server_id, raw_id.to_string());
        if self.players.contains_key(&key) {
            return Some(raw_id.to_string());
        }
        // 2. 通过 alt_id 回退
        if let Some(steam) = self.alt_id_to_steam.get(&key) {
            if self.players.contains_key(&(server_id, steam.clone())) {
                return Some(steam.clone());
            }
        }
        // 3. 通过 name 回退
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

    fn mark_apologized(&mut self, server_id: i32, steam_id: &str) -> bool {
        let key = (server_id, steam_id.to_string());
        if let Some(entry) = self.players.get_mut(&key) {
            if entry.apology_deadline.is_some() && !entry.kicked {
                entry.apologized = true;
                entry.apology_deadline = None;
                entry.last_apology_at = Some(Instant::now());
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

    /// 检查玩家是否在道歉预窗口内（刚道歉过，不需要再次道歉）
    fn in_apology_pre_window(&self, server_id: i32, steam_id: &str, window_secs: u64) -> bool {
        if let Some(entry) = self.players.get(&(server_id, steam_id.to_string())) {
            if let Some(last_apology) = entry.last_apology_at {
                return last_apology.elapsed().as_secs() < window_secs;
            }
        }
        false
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
                            ref attacker_name, ref attacker_eos, ref attacker_steam64,
                            ref victim_name, damage,
                            ref weapon, ref event_type, ..
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
                                    if event_type == "damage" || event_type == "wound" {
                                        // 注册 EOS->Steam 映射，用于道歉时回退匹配
                                        if !attacker_eos.is_empty() && !attacker_steam64.is_empty() {
                                            let mut t = tracker.write().await;
                                            t.register_alt_id(server_id, attacker_eos, attacker_steam64);
                                            drop(t);
                                        }
                                        // 标记数据库中对应的 kill_events 为 teamkill
                                        let _ = sqlx::query(
                                            "UPDATE kill_events SET is_teamkill = true \
                                             WHERE id = (SELECT id FROM kill_events \
                                             WHERE server_id = $1 AND attacker_steam64 = $2 AND victim_name = $3 \
                                             AND is_teamkill = false AND logged_at > NOW() - INTERVAL '30 seconds' \
                                             ORDER BY logged_at DESC LIMIT 1)"
                                        ).bind(server_id).bind(attacker_steam64).bind(victim).execute(&pool).await;
                                        // 友方误伤
                                        handle_teamkill(
                                            &pool, &tracker, &server_states, server_id,
                                            attacker, attacker_steam64, victim, damage,
                                            attacker_pid, &rcon_pool,
                                        ).await;
                                    }
                                    // event_type == "death" 且 same_team: TK击杀在damage阶段已处理，跳过
                                }
                                Some(false) => {
                                    if event_type == "damage" {
                                        // 敌方伤害
                                        handle_enemy_damage(
                                            &pool, server_id,
                                            attacker, attacker_steam64, victim, damage, weapon,
                                            &server_states, &rcon_pool,
                                        ).await;
                                    } else if event_type == "death" || event_type == "wound" {
                                        // 击杀/击倒通知
                                        handle_kill_notify(
                                            &pool, server_id,
                                            attacker, attacker_steam64, victim, damage, weapon,
                                            &server_states, &rcon_pool,
                                        ).await;
                                    }
                                }
                                None => {
                                    // 无法判断队伍（玩家不在缓存中），跳过
                                    tracing::debug!(server_id, attacker = %attacker, victim = %victim, "无法从缓存判断敌友，跳过伤害通知");
                                }
                            }
                        }
                    }

                    // 检测道歉消息（传递 player_name 用于回退匹配）
                    if let Some((player_name, steam_id, message)) = parse_chat(raw) {
                        process_apology(&pool, &tracker, server_id, &steam_id, &player_name, &message, &rcon_pool).await;
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

/// 处理敌方伤害通知 — 使用模板和 dispatch_message
async fn handle_enemy_damage(
    pool: &PgPool, server_id: i32,
    attacker: &str, attacker_steam64: &str,
    victim: &str, damage: f64, weapon: &str,
    server_states: &Arc<tokio::sync::RwLock<HashMap<String, serde_json::Value>>>,
    rcon_pool: &RconPool,
) {
    // 读取配置：notify_damage 开关 + hit_layout + message_mode
    let config = match sqlx::query_as::<_, (bool, String, String)>(
        "SELECT notify_damage, hit_layout, message_mode FROM damage_notify_settings WHERE server_id = $1"
    ).bind(server_id).fetch_optional(pool).await {
        Ok(Some(c)) => c,
        _ => return,
    };
    let (notify_damage, hit_layout, message_mode) = config;
    if !notify_damage { return; }

    let damage_str = format!("{:.0}", damage);
    let message = render_template(&hit_layout, &[
        ("attacker", attacker),
        ("victim", victim),
        ("damage", &damage_str),
        ("weapon", weapon),
    ]);

    dispatch_message(pool, server_id, &message, &message_mode, attacker_steam64, victim, server_states, rcon_pool).await;
}

/// 处理击杀/击倒通知（敌方）
async fn handle_kill_notify(
    pool: &PgPool, server_id: i32,
    attacker: &str, attacker_steam64: &str,
    victim: &str, damage: f64, weapon: &str,
    server_states: &Arc<tokio::sync::RwLock<HashMap<String, serde_json::Value>>>,
    rcon_pool: &RconPool,
) {
    // 读取配置：notify_kill 开关 + kill_layout + message_mode
    let config = match sqlx::query_as::<_, (bool, String, String)>(
        "SELECT notify_kill, kill_layout, message_mode FROM damage_notify_settings WHERE server_id = $1"
    ).bind(server_id).fetch_optional(pool).await {
        Ok(Some(c)) => c,
        _ => return,
    };
    let (notify_kill, kill_layout, message_mode) = config;
    if !notify_kill { return; }

    let damage_str = format!("{:.0}", damage);
    let message = render_template(&kill_layout, &[
        ("attacker", attacker),
        ("victim", victim),
        ("damage", &damage_str),
        ("weapon", weapon),
    ]);

    dispatch_message(pool, server_id, &message, &message_mode, attacker_steam64, victim, server_states, rcon_pool).await;
}

/// 处理友方误伤 — 使用模板发送消息并启动道歉倒计时
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
    // 自伤不需要道歉
    if attacker == victim {
        return;
    }

    // 查询 TK 设置
    let tk_config = match sqlx::query_as::<_, (bool, i32, String, i32, String, String, String)>(
        "SELECT enabled, apology_time_minutes, apology_keyword, apology_pre_window_secs, \
         tk_attacker_msg, tk_victim_msg, tk_broadcast_msg \
         FROM tk_settings WHERE server_id = $1"
    ).bind(server_id).fetch_optional(pool).await {
        Ok(Some(c)) => c,
        _ => return,
    };
    let (tk_enabled, apology_minutes, apology_keyword, pre_window_secs,
         tk_attacker_msg, tk_victim_msg, tk_broadcast_msg) = tk_config;
    if !tk_enabled { return; }

    let pid_str = attacker_pid.map(|id| id.to_string()).unwrap_or_else(|| attacker.to_string());
    let seconds_str = (apology_minutes * 60).to_string();
    let damage_str = format!("{:.0}", damage);

    // 模板变量
    let vars: Vec<(&str, &str)> = vec![
        ("attacker", attacker),
        ("victim", victim),
        ("damage", &damage_str),
        ("seconds", &seconds_str),
        ("keyword", &apology_keyword),
    ];

    // 1. 广播消息
    let broadcast_msg = render_template(&tk_broadcast_msg, &vars);
    if !broadcast_msg.is_empty() {
        send_rcon_cmd(pool, server_id, &format!("AdminBroadcast {}", broadcast_msg), rcon_pool).await;
    }

    // 2. 攻击者私发警告
    let attacker_msg = render_template(&tk_attacker_msg, &vars);
    if !attacker_msg.is_empty() {
        let cmd = format!("AdminWarn {} {}", pid_str, attacker_msg);
        send_rcon_cmd(pool, server_id, &cmd, rcon_pool).await;
    }

    // 3. 受害者私发警告（新增）
    let victim_msg = render_template(&tk_victim_msg, &vars);
    if !victim_msg.is_empty() {
        let states = server_states.read().await;
        let victim_pid = find_player_id(&states, server_id, victim);
        drop(states);
        if let Some(vpid) = victim_pid {
            let cmd = format!("AdminWarn {} {}", vpid, victim_msg);
            send_rcon_cmd(pool, server_id, &cmd, rcon_pool).await;
        }
    }

    // 4. 道歉预窗口检查
    let skip_apology = {
        let t = tracker.read().await;
        t.in_apology_pre_window(server_id, attacker_steam64, pre_window_secs as u64)
    };

    if skip_apology {
        tracing::info!(server_id, player = %attacker, victim = %victim, "道歉预窗口内，跳过踢出计时器");
        return;
    }

    // 5. 更新 TK 计数并重置道歉状态（建立 name->steam_id 反向索引）
    let (tk_count, timer_gen) = {
        let mut t = tracker.write().await;
        t.record_tk(server_id, attacker_steam64, attacker)
    };
    tracing::info!(server_id, player = %attacker, tk_count, victim = %victim, damage, timer_gen, "误伤事件");

    // 6. 启动道歉倒计时
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
    player_name: &str,
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
    // 先尝试直接用聊天中解析的 steam_id 匹配
    if t.mark_apologized(server_id, steam_id) {
        tracing::info!(server_id, %steam_id, %player_name, "玩家已道歉（直接匹配），取消踢出");
        drop(t);
        let msg = format!("AdminBroadcast 玩家 {} 道歉成功，已取消踢出", player_name);
        send_rcon_cmd(pool, server_id, &msg, rcon_pool).await;
        return;
    }
    // 直接匹配失败，通过 resolve_steam_id 回退查找（name / alt_id）
    if let Some(resolved_steam) = t.resolve_steam_id(server_id, steam_id, player_name) {
        if resolved_steam != steam_id && t.mark_apologized(server_id, &resolved_steam) {
            tracing::info!(server_id, %steam_id, resolved = %resolved_steam, %player_name, "玩家已道歉（回退匹配），取消踢出");
            drop(t);
            let msg = format!("AdminBroadcast 玩家 {} 道歉成功，已取消踢出", player_name);
            send_rcon_cmd(pool, server_id, &msg, rcon_pool).await;
            return;
        }
    }
    // 均未匹配到（玩家未在 pending 状态）
    drop(t);
    tracing::debug!(server_id, %steam_id, %player_name, "收到道歉关键字但未匹配到 pending 状态");
    let msg = format!("AdminBroadcast 玩家 {} 道歉失败，当前没有需要道歉的误伤记录", player_name);
    send_rcon_cmd(pool, server_id, &msg, rcon_pool).await;
}

// ═══ RCON 辅助函数（通过连接池复用连接） ═══

async fn send_rcon_cmd(_pool: &PgPool, server_id: i32, cmd: &str, rcon_pool: &RconPool) {
    match rcon_pool.execute_by_server_id(server_id, cmd).await {
        Ok(_) => {}
        Err(e) => tracing::warn!(server_id, %e, "RCON 命令执行失败"),
    }
}
