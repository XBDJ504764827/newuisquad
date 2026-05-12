use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

/// 事件类型常量
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    // RCON 事件
    RconChatMessage,
    RconPlayerWarned,
    RconPlayerKicked,
    RconPlayerBanned,
    RconPossessedAdminCamera,
    RconUnpossessedAdminCamera,
    RconSquadCreated,
    RconServerInfo,

    // 日志事件
    LogAdminBroadcast,
    LogDeployableDamaged,
    LogPlayerConnected,
    LogPlayerDamaged,
    LogPlayerDied,
    LogPlayerWounded,
    LogPlayerRevived,
    LogPlayerPossess,
    LogPlayerDisconnected,
    LogJoinSucceeded,
    LogTickRate,
    LogGameEventUnified,

    // 玩家追踪事件
    PlayerConnected,
    PlayerDisconnected,
    PlayerListUpdated,
    PlayerTeamChanged,
    PlayerSquadChanged,
    SquadCreated,
    SquadDisbanded,
    EnhancedTeamkill,
    PlayerStatsUpdated,

    // 通用
    All,
}

impl EventType {
    pub fn is_rcon_event(&self) -> bool {
        matches!(
            self,
            EventType::RconChatMessage
                | EventType::RconPlayerWarned
                | EventType::RconPlayerKicked
                | EventType::RconPlayerBanned
                | EventType::RconPossessedAdminCamera
                | EventType::RconUnpossessedAdminCamera
                | EventType::RconSquadCreated
                | EventType::RconServerInfo
        )
    }

    pub fn is_log_event(&self) -> bool {
        !self.is_rcon_event() && !self.is_player_tracker_event()
    }

    pub fn is_player_tracker_event(&self) -> bool {
        matches!(
            self,
            EventType::PlayerConnected
                | EventType::PlayerDisconnected
                | EventType::PlayerListUpdated
                | EventType::PlayerTeamChanged
                | EventType::PlayerSquadChanged
                | EventType::SquadCreated
                | EventType::SquadDisbanded
                | EventType::EnhancedTeamkill
                | EventType::PlayerStatsUpdated
        )
    }
}

/// 统一事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_log: Option<String>,
}

impl Event {
    pub fn new(event_type: EventType, data: serde_json::Value, raw_log: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            timestamp: Utc::now(),
            data,
            raw_log,
        }
    }
}

/// 事件过滤器
#[derive(Debug, Clone)]
pub struct EventFilter {
    pub types: Vec<EventType>,
}

impl EventFilter {
    pub fn all() -> Self {
        Self { types: vec![EventType::All] }
    }

    pub fn by_types(types: Vec<EventType>) -> Self {
        Self { types }
    }

    pub fn matches(&self, event: &Event) -> bool {
        if self.types.len() == 1 && self.types[0] == EventType::All {
            return true;
        }
        if self.types.is_empty() {
            return true;
        }
        self.types.contains(&event.event_type)
    }
}

/// 事件订阅者
#[derive(Debug)]
pub struct EventSubscriber {
    pub id: Uuid,
    pub channel: mpsc::Receiver<Event>,
    pub filter: EventFilter,
}

/// 事件管理器
pub struct EventManager {
    subscribers: Arc<RwLock<HashMap<Uuid, mpsc::Sender<Event>>>>,
    filters: Arc<RwLock<HashMap<Uuid, EventFilter>>>,
    event_queue: mpsc::Sender<Event>,
    buffer_size: usize,
}

impl EventManager {
    pub fn new(buffer_size: usize) -> Self {
        let (tx, rx) = mpsc::channel::<Event>(buffer_size.max(256));

        let subscribers: Arc<RwLock<HashMap<Uuid, mpsc::Sender<Event>>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let filters: Arc<RwLock<HashMap<Uuid, EventFilter>>> =
            Arc::new(RwLock::new(HashMap::new()));

        let subs_clone = subscribers.clone();
        let filters_clone = filters.clone();

        tokio::spawn(async move {
            let mut rx = rx;
            while let Some(event) = rx.recv().await {
                let subs = subs_clone.read().await;
                let filt = filters_clone.read().await;
                for (id, sender) in subs.iter() {
                    if let Some(filter) = filt.get(id) {
                        if filter.matches(&event) {
                            if let Err(e) = sender.send(event.clone()).await {
                                tracing::warn!("事件发送到订阅者 {} 失败: {}", id, e);
                            }
                        }
                    }
                }
            }
            tracing::info!("事件处理器停止");
        });

        Self {
            subscribers,
            filters,
            event_queue: tx,
            buffer_size,
        }
    }

    /// 发布事件
    pub fn publish(&self, event: Event) {
        if let Err(e) = self.event_queue.try_send(event) {
            tracing::warn!("事件队列已满，丢弃事件: {}", e);
        }
    }

    /// 订阅事件
    pub async fn subscribe(&self, filter: EventFilter) -> EventSubscriber {
        let id = Uuid::new_v4();
        let (tx, rx) = mpsc::channel::<Event>(self.buffer_size.max(100));

        self.subscribers.write().await.insert(id, tx);
        self.filters.write().await.insert(id, filter.clone());

        tracing::info!("新订阅者注册: {}", id);

        EventSubscriber {
            id,
            channel: rx,
            filter,
        }
    }

    /// 取消订阅
    pub async fn unsubscribe(&self, subscriber_id: Uuid) {
        self.subscribers.write().await.remove(&subscriber_id);
        self.filters.write().await.remove(&subscriber_id);
        tracing::info!("订阅者取消: {}", subscriber_id);
    }

    /// 获取统计信息
    pub async fn stats(&self) -> HashMap<String, usize> {
        let subs = self.subscribers.read().await;
        let mut stats = HashMap::new();
        stats.insert("subscribers".to_string(), subs.len());
        stats.insert("queue_capacity".to_string(), self.buffer_size);
        stats
    }
}