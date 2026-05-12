use crate::event_manager::{Event, EventType, EventManager};
use crate::player_tracker::PlayerTracker;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// 封禁条目
#[derive(Debug, Clone)]
pub struct BanEntry {
    pub steam_id: String,
    pub reason: String,
    pub permanent: bool,
    pub expires_at: Option<i64>,
}

/// 封禁执行器
pub struct BanEnforcer {
    bans: Arc<RwLock<HashSet<String>>>,
    ban_details: Arc<RwLock<HashMap<String, BanEntry>>>,
    rcon_cmd_tx: mpsc::Sender<String>,
    event_manager: Arc<EventManager>,
    _player_tracker: Arc<PlayerTracker>,
}

impl BanEnforcer {
    pub fn new(
        rcon_cmd_tx: mpsc::Sender<String>,
        event_manager: Arc<EventManager>,
        player_tracker: Arc<PlayerTracker>,
    ) -> Self {
        Self {
            bans: Arc::new(RwLock::new(HashSet::new())),
            ban_details: Arc::new(RwLock::new(HashMap::new())),
            rcon_cmd_tx,
            event_manager,
            _player_tracker: player_tracker,
        }
    }

    /// 启动封禁执行器
    pub async fn start(&self) {
        let filter = crate::event_manager::EventFilter::by_types(vec![
            EventType::PlayerConnected,
            EventType::LogPlayerConnected,
        ]);

        let subscriber = self.event_manager.subscribe(filter).await;
        let sub_id = subscriber.id;

        tracing::info!("封禁执行器启动，订阅 ID: {}", sub_id);

        let bans = self.bans.clone();
        let ban_details = self.ban_details.clone();
        let rcon_cmd_tx = self.rcon_cmd_tx.clone();

        tokio::spawn(async move {
            let mut channel = subscriber.channel;
            while let Some(event) = channel.recv().await {
                handle_ban_check(&event, &bans, &ban_details, &rcon_cmd_tx).await;
            }
            tracing::info!("封禁执行器事件监听停止");
        });
    }

    /// 加载封禁列表
    pub async fn load_bans(&self, entries: Vec<BanEntry>) {
        let mut bans = self.bans.write().await;
        let mut ban_details = self.ban_details.write().await;

        bans.clear();
        ban_details.clear();

        for entry in entries {
            bans.insert(entry.steam_id.clone());
            ban_details.insert(entry.steam_id.clone(), entry);
        }

        tracing::info!("封禁列表已加载: {} 条", bans.len());
    }

    /// 添加封禁
    pub async fn add_ban(&self, steam_id: String, reason: String, permanent: bool) {
        let entry = BanEntry {
            steam_id: steam_id.clone(),
            reason,
            permanent,
            expires_at: None,
        };
        self.bans.write().await.insert(steam_id.clone());
        self.ban_details.write().await.insert(steam_id, entry);
    }

    /// 移除封禁
    pub async fn remove_ban(&self, steam_id: &str) {
        self.bans.write().await.remove(steam_id);
        self.ban_details.write().await.remove(steam_id);
    }

    /// 检查是否被封禁
    pub async fn is_banned(&self, steam_id: &str) -> bool {
        self.bans.read().await.contains(steam_id)
    }

    /// 获取封禁详情
    pub async fn get_ban_details(&self, steam_id: &str) -> Option<BanEntry> {
        self.ban_details.read().await.get(steam_id).cloned()
    }

    /// 从本地文件加载封禁列表
    pub async fn load_from_file(&self, path: &str) {
        if let Ok(content) = std::fs::read_to_string(path) {
            let mut entries = Vec::new();
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') { continue; }
                let parts: Vec<&str> = line.split('/').collect();
                if parts.len() >= 2 {
                    let steam_part = parts[0].trim();
                    let steam_id = if steam_part.starts_with("SteamID:") {
                        steam_part[8..].trim().to_string()
                    } else {
                        crate::rcon_listener::find_steam64_in_line(steam_part)
                    };

                    if !steam_id.is_empty() && steam_id.starts_with("7656119") {
                        let reason = parts[1].trim().to_string();
                        let permanent = parts.len() < 3 || parts[2].trim() == "0";
                        entries.push(BanEntry { steam_id, reason, permanent, expires_at: None });
                    }
                }
            }
            self.load_bans(entries).await;
        }
    }
}

/// 处理封禁检查
async fn handle_ban_check(
    event: &Event,
    bans: &Arc<RwLock<HashSet<String>>>,
    ban_details: &Arc<RwLock<HashMap<String, BanEntry>>>,
    rcon_cmd_tx: &mpsc::Sender<String>,
) {
    let steam_id = match event.event_type {
        EventType::PlayerConnected | EventType::LogPlayerConnected => {
            event.data.get("steam_id").and_then(|v| v.as_str()).unwrap_or("").to_string()
        }
        _ => return,
    };

    if steam_id.is_empty() { return; }

    let bans_r = bans.read().await;
    if bans_r.contains(&steam_id) {
        let details = ban_details.read().await;
        let reason = details.get(&steam_id)
            .map(|e| e.reason.clone())
            .unwrap_or_else(|| "You are banned from this server".to_string());

        let kick_cmd = format!("AdminKick {} {}", steam_id, reason);
        if let Err(e) = rcon_cmd_tx.send(kick_cmd).await {
            tracing::error!("封禁踢出命令发送失败: {}", e);
        } else {
            tracing::info!("封禁执行: 踢出 {}", steam_id);
        }
    }
}