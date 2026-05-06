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
//  伤害通知后台服务
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
                            ref victim_name, damage,
                            is_kill, is_teamkill, ..
                        } = event {
                            // 队友伤害交由误杀检测服务处理
                            if is_teamkill {
                                continue;
                            }

                            let settings = match sqlx::query_as::<_, (bool, bool, bool)>(
                                "SELECT enabled, notify_kill, notify_damage FROM damage_notify_settings WHERE server_id = $1"
                            ).bind(server_id).fetch_optional(&pool).await {
                                Ok(Some(s)) => s,
                                _ => continue,
                            };
                            let (enabled, notify_kill, notify_damage) = settings;
                            if !enabled { continue; }

                            let cmd = build_notify_command(
                                attacker_name, victim_name, damage,
                                is_kill, notify_kill, notify_damage,
                            );
                            let Some(cmd) = cmd else { continue };

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

                            send_rcon_cmd(&pool, server_id, &cmd).await;
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

fn build_notify_command(
    attacker: &str, victim: &str, damage: f64,
    is_kill: bool, notify_kill: bool, notify_damage: bool,
) -> Option<String> {
    // 击倒通知: AdminWarn "击倒了<被击倒玩家>"
    if is_kill && notify_kill {
        return Some(format!("AdminWarn \"{}\" \"击倒了{}\"", attacker, victim));
    }
    // 伤害通知: AdminWarn "对<被造成伤害玩家>造成了<伤害数值>点伤害"
    if !is_kill && notify_damage {
        return Some(format!("AdminWarn \"{}\" \"对{}造成了{:.0}点伤害\"", attacker, victim, damage));
    }
    None
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
