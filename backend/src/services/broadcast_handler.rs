use std::collections::HashMap;
use sqlx::PgPool;
use tokio::time::{sleep, Duration};
use crate::models::server_log::LogEntry;
use crate::rcon_client::squad::SquadRcon;

/// 从日志行解析玩家进入事件
fn parse_player_join(line: &str) -> Option<(String, String)> {
    let lower = line.to_lowercase();
    if !lower.contains("join") || lower.contains("left") || lower.contains("disconnect") {
        return None;
    }
    let mut player_name = String::new();
    let mut steam_id = String::new();

    if let Some(quote_start) = line.find('"') {
        let rest = &line[quote_start + 1..];
        if let Some(quote_end) = rest.find('"') {
            player_name = rest[..quote_end].to_string();
            let after_name = &rest[quote_end + 1..];
            if let Some(paren_start) = after_name.find('(') {
                let inner = &after_name[paren_start + 1..];
                if let Some(paren_end) = inner.find(')') {
                    let id = &inner[..paren_end];
                    if id.chars().all(|c| c.is_ascii_digit()) && id.len() >= 10 {
                        steam_id = id.to_string();
                    }
                }
            }
        }
    }

    if !player_name.is_empty() && !steam_id.is_empty() {
        Some((player_name, steam_id))
    } else {
        None
    }
}

/// 从日志行解析聊天消息
fn parse_chat(line: &str) -> Option<(String, String, String)> {
    if !line.contains("[Chat]") && !line.to_lowercase().contains("chat") {
        return None;
    }
    let content = if let Some(pos) = line.find("[Chat]") {
        line[pos + 6..].trim()
    } else if let Some(pos) = line.to_lowercase().find("chat:") {
        &line[pos + 5..].trim()
    } else {
        return None;
    };

    if let Some(colon_pos) = content.find(": ") {
        let header = &content[..colon_pos];
        let message = content[colon_pos + 2..].trim().to_string();

        if let Some(paren_start) = header.rfind('(') {
            if let Some(paren_end) = header.rfind(')') {
                let sid = &header[paren_start + 1..paren_end];
                let name = header[..paren_start].trim().to_string();
                if sid.chars().all(|c| c.is_ascii_digit()) && sid.len() >= 10 {
                    return Some((name, sid.to_string(), message));
                }
            }
        }
    }
    None
}

pub fn start_broadcast_handler(
    pool: PgPool,
    mut log_rx: tokio::sync::broadcast::Receiver<LogEntry>,
) -> tokio::task::JoinHandle<()> {
    tracing::info!("广播处理服务已启动");

    // 定时通告循环
    let pool_for_timer = pool.clone();
    tokio::spawn(async move {
        let mut last_sent: HashMap<i32, chrono::DateTime<chrono::Utc>> = HashMap::new();
        loop {
            sleep(Duration::from_secs(30)).await;
            let now = chrono::Utc::now();

            let rows = match sqlx::query_as::<_, (i32, String, i32, String, bool, String, i32)>(
                "SELECT s.id, s.ip, s.rcon_port, s.rcon_password, bc.announcement_enabled, bc.announcement_content, bc.announcement_interval FROM servers s JOIN broadcast_settings bc ON s.id = bc.server_id"
            ).fetch_all(&pool_for_timer).await {
                Ok(r) => r,
                Err(_) => continue,
            };

            for (sid, ip, rcon_port, rcon_pass, bc_enabled, bc_content, bc_interval) in &rows {
                if *bc_enabled && !bc_content.is_empty() && *bc_interval > 0 {
                    let entry = last_sent.entry(-*sid).or_insert_with(|| now - chrono::Duration::minutes(*bc_interval as i64));
                    let elapsed = now - *entry;
                    if elapsed.num_minutes() >= *bc_interval as i64 {
                        let cmd = format!("AdminBroadcast \"{}\"", bc_content);
                        if let Err(e) = send_rcon(&ip, *rcon_port as u16, &rcon_pass, &cmd).await {
                            tracing::error!(server_id = *sid, error = %e, "定时通告发送失败");
                        } else {
                            tracing::info!(server_id = *sid, "已发送定时通告");
                        }
                        *entry = now;
                    }
                }
            }

            // 处理 announcements 多条通告
            let ann_configs = match sqlx::query_as::<_, (i32, String, i32, String)>(
                "SELECT DISTINCT a.server_id, s.ip, s.rcon_port, s.rcon_password FROM announcements a JOIN servers s ON s.id = a.server_id WHERE a.enabled = true"
            ).fetch_all(&pool_for_timer).await {
                Ok(r) => r,
                Err(_) => continue,
            };

            for (sid, ip, rcon_port, rcon_pass) in &ann_configs {
                let entry = last_sent.entry(*sid).or_insert_with(|| now - chrono::Duration::minutes(5));
                let elapsed = now - *entry;
                if elapsed.num_minutes() < 5 {
                    continue;
                }

                if let Ok(anns) = sqlx::query_as::<_, (i32, String, i32)>(
                    "SELECT id, content, interval_minutes FROM announcements WHERE server_id=$1 AND enabled=true"
                ).bind(sid).fetch_all(&pool_for_timer).await {
                    for (_id, content, interval) in &anns {
                        if *interval > 0 {
                            let cmd = format!("AdminBroadcast \"{}\"", content);
                            let _ = send_rcon(&ip, *rcon_port as u16, &rcon_pass, &cmd).await;
                        }
                    }
                }
                *entry = now;
            }
        }
    });

    // 主循环：处理日志事件（进入提醒、OP列表、自动回复）
    let runtime_pool = pool.clone();
    tokio::spawn(async move {
        loop {
            match log_rx.recv().await {
                Ok(entry) => {
                    let raw = entry.raw_line.as_deref().unwrap_or(&entry.message);
                    let server_id = entry.server_id;
                    if server_id == 0 { continue; }

                    let bc_config = match sqlx::query_as::<_, (String, i32, String, bool, String, bool, String)>(
                        "SELECT s.ip, s.rcon_port, s.rcon_password, bc.join_message_enabled, bc.join_message, bc.gameop_list_enabled, bc.gameop_list_message FROM servers s JOIN broadcast_settings bc ON s.id = bc.server_id WHERE s.id = $1"
                    ).bind(server_id).fetch_optional(&runtime_pool).await {
                        Ok(Some(c)) => c,
                        Ok(None) => continue,
                        Err(_) => continue,
                    };

                    let (ip, rcon_port, rcon_pass, join_enabled, join_msg, op_enabled, op_msg) = bc_config;

                    // 1. 玩家进入提醒
                    if join_enabled {
                        if let Some((player_name, _)) = parse_player_join(raw) {
                            let welcome = join_msg.replace("{player}", &player_name);
                            let cmd = format!("AdminBroadcast \"{}\"", welcome);
                            match send_rcon(&ip, rcon_port as u16, &rcon_pass, &cmd).await {
                                Ok(_) => tracing::info!(server_id, player = %player_name, "已发送欢迎消息"),
                                Err(e) => tracing::error!(server_id, error = %e, "欢迎消息发送失败"),
                            }
                        }
                    }

                    // 2. 在线OP列表 & 自动回复
                    // 优先用 Agent 已解析的干净格式 (category=Chat-*, message="玩家名: 消息")
                    let chat_info = if let Some(ref cat) = entry.category {
                        if cat.starts_with("Chat-") {
                            entry.message.split_once(": ").map(|(name, msg)| (name.to_string(), String::new(), msg.to_string()))
                        } else {
                            parse_chat(raw)
                        }
                    } else {
                        parse_chat(raw)
                    };
                    if let Some((player_name, _steam_id, message)) = chat_info {
                        // OP 列表关键字检测
                        if op_enabled {
                            let lower = message.to_lowercase();
                            if lower.contains("op") || lower.contains("管理员") || lower.contains("管理") {
                                match sqlx::query_as::<_, (String,)>(
                                    "SELECT username FROM admin_users WHERE is_active = true ORDER BY id"
                                ).fetch_all(&runtime_pool).await {
                                    Ok(admins) => {
                                        let list: Vec<String> = admins.into_iter().map(|(u,)| u).collect();
                                        let oplist = if list.is_empty() { "当前暂无在线管理员".to_string() } else { list.join(", ") };
                                        let reply = op_msg.replace("{oplist}", &oplist);
                                        let cmd = format!("AdminWarn \"{}\" \"{}\"", player_name, reply);
                                        let _ = send_rcon(&ip, rcon_port as u16, &rcon_pass, &cmd).await;
                                        tracing::info!(server_id, player = %player_name, "已回复OP列表");
                                    }
                                    Err(_) => {}
                                }
                            }
                        }

                        // 自动回复规则
                        if let Ok(replies) = sqlx::query_as::<_, (String, String)>(
                            "SELECT keyword, reply_message FROM auto_replies WHERE server_id=$1 AND enabled=true"
                        ).bind(server_id).fetch_all(&runtime_pool).await {
                            let lower_msg = message.to_lowercase();
                            for (keyword, reply_message) in &replies {
                                if lower_msg.contains(&keyword.to_lowercase()) {
                                    let cmd = format!("AdminBroadcast \"{}\"", reply_message);
                                    let _ = send_rcon(&ip, rcon_port as u16, &rcon_pass, &cmd).await;
                                    tracing::info!(server_id, player = %player_name, keyword, "已自动回复（广播）");
                                    break;
                                }
                            }
                        }
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(skipped = n, "广播处理滞后");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    })
}

async fn send_rcon(ip: &str, port: u16, password: &str, command: &str) -> Result<(), String> {
    let mut rcon = SquadRcon::connect(ip, port, password).await?;
    rcon.execute(command).await?;
    Ok(())
}
