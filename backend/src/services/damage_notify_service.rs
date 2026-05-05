use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Instant;
use crate::models::damage_notify_settings::{DamageNotifySettings, UpdateDamageNotifyRequest};
use crate::models::server_log::LogEntry;
use crate::repositories::damage_notify_repo;
use crate::rcon_client::squad::SquadRcon;

pub async fn get(pool: &PgPool, server_id: i32) -> Result<DamageNotifySettings, sqlx::Error> {
    damage_notify_repo::get_or_create(pool, server_id).await
}
pub async fn update(pool: &PgPool, server_id: i32, req: UpdateDamageNotifyRequest) -> Result<DamageNotifySettings, sqlx::Error> {
    damage_notify_repo::update(pool, server_id, &req).await
}

// ════════════════════════════════════════════
//  伤害 / TK 通知后台服务
// ════════════════════════════════════════════

pub fn start_damage_notify(
    pool: PgPool,
    mut log_rx: tokio::sync::broadcast::Receiver<LogEntry>,
) -> tokio::task::JoinHandle<()> {
    let cooldowns: Arc<RwLock<HashMap<(i32, String, String), Instant>>> = Arc::new(RwLock::new(HashMap::new()));

    tokio::spawn(async move {
        tracing::info!("伤害通知服务已启动");

        loop {
            match log_rx.recv().await {
                Ok(entry) => {
                    let server_id = entry.server_id;
                    if server_id == 0 { continue; }

                    let raw = entry.raw_line.as_deref().unwrap_or(&entry.message);
                    if raw.is_empty() { continue; }

                    if let Some(event) = crate::services::squad_log_parser::parse_line(raw) {
                        use crate::services::squad_log_parser::ParsedEvent;
                        if let ParsedEvent::KillEvent {
                            ref attacker_name, ref attacker_steam64,
                            ref victim_name, damage, ref weapon,
                            is_kill, is_teamkill, ..
                        } = event {
                            // 查询设置
                            let settings = match sqlx::query_as::<_, (bool, f64, bool, bool, bool, f64)>(
                                "SELECT enabled, min_damage, notify_tk, notify_damage, notify_high_damage, high_damage_threshold \
                                 FROM damage_notify_settings WHERE server_id = $1"
                            ).bind(server_id).fetch_optional(&pool).await {
                                Ok(Some(s)) => s,
                                _ => continue,
                            };
                            let (enabled, min_damage, notify_tk, notify_damage, notify_high_damage, high_damage_threshold) = settings;
                            if !enabled { continue; }

                            let cmds = build_notify_commands(
                                attacker_name, victim_name, &weapon, damage, is_kill, is_teamkill,
                                min_damage, notify_tk, notify_damage, notify_high_damage, high_damage_threshold,
                            );
                            if cmds.is_empty() { continue; }

                            // 冷却：同 server+attacker+victim 10 秒内不重复
                            {
                                let mut cds = cooldowns.write().await;
                                let key = (server_id, attacker_steam64.clone(), victim_name.clone());
                                let now = Instant::now();
                                if let Some(last) = cds.get(&key) {
                                    if now.duration_since(*last).as_secs() < 10 { continue; }
                                }
                                cds.insert(key, now);
                            }

                            for cmd in &cmds {
                                send_rcon_cmd(&pool, server_id, cmd).await;
                            }
                        }
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(skipped = n, "伤害通知服务滞后");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
        tracing::info!("伤害通知服务已停止");
    })
}

fn build_notify_commands(
    attacker: &str, victim: &str, weapon: &str,
    damage: f64, is_kill: bool, is_teamkill: bool,
    min_damage: f64, notify_tk: bool, notify_damage: bool,
    notify_high_damage: bool, high_damage_threshold: f64,
) -> Vec<String> {
    let aname = trunc(attacker);
    let vname = trunc(victim);
    let wname = simplify_weapon(weapon);

    if is_teamkill && notify_tk {
        if is_kill {
            // TK 致死：全员广播 + 对 TK 者发送黄字警告
            return vec![
                format!("AdminBroadcast \"💀 队友击杀! {} 击杀了队友 {}\"", aname, vname),
                format!("AdminWarn \"{}\" \"你击杀了队友 {}！这是严重的团队误杀行为\"", attacker, vname),
            ];
        } else {
            // TK 误伤：对攻击者发送黄字警告 + 全员广播
            return vec![
                format!("AdminWarn \"{}\" \"你攻击了队友 {}！\"", attacker, vname),
                format!("AdminBroadcast \"⚠️ 队友误伤: {} 误伤了 {} ({} {:.0}伤害)\"", aname, vname, wname, damage),
            ];
        }
    }
    if is_kill && !is_teamkill && notify_high_damage && damage >= high_damage_threshold {
        return vec![
            format!("AdminBroadcast \"💥 {} 击杀了 {} ({} {:.0}伤害)\"", aname, vname, wname, damage),
        ];
    }
    if !is_teamkill && notify_damage && damage >= min_damage {
        return vec![
            format!("AdminBroadcast \"🔫 {} → {} ({} {:.0}伤害)\"", aname, vname, wname, damage),
        ];
    }
    vec![]
}

async fn send_rcon_cmd(pool: &PgPool, server_id: i32, cmd: &str) {
    let creds = match sqlx::query_as::<_, (String, i32, String)>(
        "SELECT ip, rcon_port, rcon_password FROM servers WHERE id = $1"
    ).bind(server_id).fetch_optional(pool).await {
        Ok(Some(c)) => c,
        _ => return,
    };
    let (_ip, rcon_port, rcon_password) = creds;
    if rcon_password.is_empty() { return; }

    match SquadRcon::connect(&_ip, rcon_port as u16, &rcon_password).await {
        Ok(mut rcon) => { let _ = rcon.execute(cmd).await; }
        Err(e) => tracing::warn!(server_id, %e, "通知 RCON 连接失败"),
    }
}

fn trunc(s: &str) -> String {
    let s = s.trim();
    if s.chars().count() > 16 { format!("{}..", s.chars().take(14).collect::<String>()) } else { s.to_string() }
}

fn simplify_weapon(s: &str) -> String {
    let s = s.trim().strip_prefix("BP_").unwrap_or(s).strip_suffix("_C").unwrap_or(s);
    if let Some(pos) = s.rfind('_') {
        let tail = &s[pos+1..];
        if tail.chars().all(|c| c.is_ascii_digit()) && tail.len() > 5 { return s[..pos].to_string(); }
    }
    s.to_string()
}
