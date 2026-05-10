use std::collections::HashMap;
use std::sync::Arc;
use sqlx::PgPool;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use crate::models::server_log::LogEntry;
use crate::rcon_client::pool::RconPool;
use crate::services::chat_automod::ChatAutomod;

// ════════════════════════════════════════════
//  广播配置缓存 — 避免每次日志事件查询 DB
// ════════════════════════════════════════════

/// 每个服务器的广播设置缓存
#[derive(Clone, Default)]
struct BcSetting {
    join_enabled: bool,
    join_msg: String,
    op_enabled: bool,
    op_msg: String,
    ann_enabled: bool,
    ann_content: String,
    ann_interval: i32,
}

/// 内存缓存，定期从 DB 刷新，热路径零 DB 查询
#[derive(Clone, Default)]
struct BroadcastCache {
    settings: HashMap<i32, BcSetting>,
    auto_replies: HashMap<i32, Vec<(String, String)>>,
    announcements: HashMap<i32, Vec<(i32, String, i32)>>,
    admin_users: Vec<String>,
}

impl BroadcastCache {
    /// 从数据库刷新全部缓存
    async fn refresh(pool: &PgPool) -> Self {
        let mut cache = BroadcastCache::default();

        // 1. broadcast_settings（合并 JOIN）
        if let Ok(rows) = sqlx::query_as::<_, (i32, bool, String, bool, String, bool, String, i32)>(
            "SELECT s.id, bc.join_message_enabled, bc.join_message, \
             bc.gameop_list_enabled, bc.gameop_list_message, \
             bc.announcement_enabled, bc.announcement_content, bc.announcement_interval \
             FROM servers s JOIN broadcast_settings bc ON s.id = bc.server_id"
        ).fetch_all(pool).await {
            for (sid, je, jm, oe, om, ae, ac, ai) in rows {
                cache.settings.insert(sid, BcSetting {
                    join_enabled: je,
                    join_msg: jm,
                    op_enabled: oe,
                    op_msg: om,
                    ann_enabled: ae,
                    ann_content: ac,
                    ann_interval: ai,
                });
            }
        }

        // 2. 自动回复规则
        if let Ok(rows) = sqlx::query_as::<_, (i32, String, String)>(
            "SELECT server_id, keyword, reply_message FROM auto_replies WHERE enabled=true"
        ).fetch_all(pool).await {
            for (sid, kw, reply) in rows {
                cache.auto_replies.entry(sid).or_default().push((kw, reply));
            }
        }

        // 3. 多条通告
        if let Ok(rows) = sqlx::query_as::<_, (i32, i32, String, i32)>(
            "SELECT server_id, id, content, interval_minutes FROM announcements WHERE enabled=true"
        ).fetch_all(pool).await {
            for (sid, id, content, interval) in rows {
                cache.announcements.entry(sid).or_default().push((id, content, interval));
            }
        }

        // 4. 活跃管理员
        if let Ok(rows) = sqlx::query_as::<_, (String,)>(
            "SELECT username FROM admin_users WHERE is_active = true ORDER BY id"
        ).fetch_all(pool).await {
            cache.admin_users = rows.into_iter().map(|(u,)| u).collect();
        }

        cache
    }
}

// ════════════════════════════════════════════
//  日志解析（与原逻辑一致）
// ════════════════════════════════════════════

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

// ════════════════════════════════════════════
//  启动入口
// ════════════════════════════════════════════

pub fn start_broadcast_handler(
    pool: PgPool,
    mut log_rx: tokio::sync::broadcast::Receiver<LogEntry>,
    rcon_pool: RconPool,
    chat_automod: Arc<RwLock<ChatAutomod>>,
) -> tokio::task::JoinHandle<()> {
    tracing::info!("广播处理服务已启动");

    // 共享缓存：定时刷新，热路径只读
    let cache: Arc<RwLock<BroadcastCache>> = Arc::new(RwLock::new(BroadcastCache::default()));

    // ─── 定时任务：刷新缓存 + 处理定时通告 ───
    let pool_for_timer = pool.clone();
    let rcon_for_timer = rcon_pool.clone();
    let cache_for_timer = cache.clone();
    tokio::spawn(async move {
        let mut last_sent: HashMap<i32, chrono::DateTime<chrono::Utc>> = HashMap::new();
        loop {
            sleep(Duration::from_secs(30)).await;

            // 刷新缓存
            let new_cache = BroadcastCache::refresh(&pool_for_timer).await;
            *cache_for_timer.write().await = new_cache;
            let guard = cache_for_timer.read().await;
            let now = chrono::Utc::now();

            // 1. 处理 broadcast_settings 中的定时通告
            for (sid, s) in &guard.settings {
                if s.ann_enabled && !s.ann_content.is_empty() && s.ann_interval > 0 {
                    let entry = last_sent.entry(*sid).or_insert_with(|| now - chrono::Duration::minutes(s.ann_interval as i64));
                    let elapsed = now - *entry;
                    if elapsed.num_minutes() >= s.ann_interval as i64 {
                        let cmd = format!("AdminBroadcast \"{}\"", s.ann_content);
                        if let Err(e) = send_rcon(&rcon_for_timer, *sid, &cmd).await {
                            tracing::error!(server_id = *sid, error = %e, "定时通告发送失败");
                        } else {
                            tracing::info!(server_id = *sid, "已发送定时通告");
                        }
                        *entry = now;
                    }
                }
            }

            // 2. 处理 announcements 多条通告
            for (sid, anns) in &guard.announcements {
                let entry = last_sent.entry(*sid).or_insert_with(|| now - chrono::Duration::minutes(5));
                let elapsed = now - *entry;
                if elapsed.num_minutes() < 5 {
                    continue;
                }
                for (_id, content, interval) in anns {
                    if *interval > 0 {
                        let cmd = format!("AdminBroadcast \"{}\"", content);
                        let _ = send_rcon(&rcon_for_timer, *sid, &cmd).await;
                    }
                }
                *entry = now;
            }
        }
    });

    // ─── 主循环：处理日志事件（全部从缓存读取，零 DB 查询） ───
    let runtime_pool = pool.clone();
    let runtime_rcon = rcon_pool;
    let runtime_automod = chat_automod;
    let runtime_cache = cache;
    tokio::spawn(async move {
        loop {
            match log_rx.recv().await {
                Ok(entry) => {
                    let raw = entry.raw_line.as_deref().unwrap_or(&entry.message);
                    let server_id = entry.server_id;
                    if server_id == 0 { continue; }

                    let guard = runtime_cache.read().await;
                    let bc = match guard.settings.get(&server_id) {
                        Some(s) => s.clone(),
                        None => continue,
                    };

                    // 1. 玩家进入提醒
                    if bc.join_enabled {
                        if let Some((player_name, _)) = parse_player_join(raw) {
                            let welcome = bc.join_msg.replace("{player}", &player_name);
                            let cmd = format!("AdminBroadcast \"{}\"", welcome);
                            match send_rcon(&runtime_rcon, server_id, &cmd).await {
                                Ok(_) => tracing::info!(server_id, player = %player_name, "已发送欢迎消息"),
                                Err(e) => tracing::error!(server_id, error = %e, "欢迎消息发送失败"),
                            }
                        }
                    }

                    // 2. 聊天事件处理
                    let chat_info = if let Some(ref cat) = entry.category {
                        if cat.starts_with("Chat-") {
                            entry.message.split_once(": ").map(|(name, msg)| (name.to_string(), String::new(), msg.to_string()))
                        } else {
                            parse_chat(raw)
                        }
                    } else {
                        parse_chat(raw)
                    };
                    if let Some((player_name, ref steam_id, message)) = chat_info {
                        // 0. 聊天审核
                        {
                            let automod = runtime_automod.read().await;
                            let chat_channel = entry.category.as_deref().unwrap_or("Chat-All");
                            if let Some((filter_match, violation_count)) = automod.check_message(
                                &runtime_pool, server_id, &player_name, steam_id, &message, chat_channel,
                            ).await {
                                if let Some(action) = automod.determine_action(server_id, violation_count) {
                                    let cmd = automod.build_rcon_command(&action, &player_name, steam_id);
                                    let _ = send_rcon(&runtime_rcon, server_id, &cmd).await;
                                    tracing::info!(server_id, player = %player_name, violation = violation_count,
                                        category = %filter_match.category.as_str(), word = %filter_match.matched_word,
                                        action = %action.action, "聊天审核触发");
                                    automod.record_violation(
                                        &runtime_pool, server_id, steam_id, &player_name, &message,
                                        filter_match.category.as_str(), &filter_match.matched_word, &action.action,
                                    ).await;
                                }
                            }
                        }

                        // OP 列表关键字检测
                        if bc.op_enabled {
                            let lower = message.to_lowercase();
                            if lower.contains("op") || lower.contains("管理员") || lower.contains("管理") {
                                let oplist = if guard.admin_users.is_empty() {
                                    "当前暂无在线管理员".to_string()
                                } else {
                                    guard.admin_users.join(", ")
                                };
                                let reply = bc.op_msg.replace("{oplist}", &oplist);
                                let cmd = format!("AdminWarn \"{}\" \"{}\"", player_name, reply);
                                let _ = send_rcon(&runtime_rcon, server_id, &cmd).await;
                                tracing::info!(server_id, player = %player_name, "已回复OP列表");
                            }
                        }

                        // 自动回复规则（从缓存读取）
                        if let Some(replies) = guard.auto_replies.get(&server_id) {
                            let lower_msg = message.to_lowercase();
                            for (keyword, reply_message) in replies {
                                if lower_msg.contains(&keyword.to_lowercase()) {
                                    let cmd = format!("AdminBroadcast \"{}\"", reply_message);
                                    let _ = send_rcon(&runtime_rcon, server_id, &cmd).await;
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

async fn send_rcon(pool: &RconPool, server_id: i32, command: &str) -> Result<(), String> {
    pool.execute_by_server_id(server_id, command).await?;
    Ok(())
}
