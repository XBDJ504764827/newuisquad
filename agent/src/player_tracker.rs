use crate::event_manager::{Event, EventType, EventManager};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 玩家状态
#[derive(Debug, Clone)]
pub struct PlayerState {
    pub eos_id: String,
    pub steam_id: String,
    pub name: String,
    pub team_id: String,
    pub squad_id: String,
    pub role: String,
    pub is_connected: bool,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// 玩家追踪器
pub struct PlayerTracker {
    players: Arc<RwLock<HashMap<String, PlayerState>>>,
    name_index: Arc<RwLock<HashMap<String, String>>>,
    steam_index: Arc<RwLock<HashMap<String, String>>>,
    event_manager: Arc<EventManager>,
}

impl PlayerTracker {
    pub fn new(event_manager: Arc<EventManager>) -> Self {
        Self {
            players: Arc::new(RwLock::new(HashMap::new())),
            name_index: Arc::new(RwLock::new(HashMap::new())),
            steam_index: Arc::new(RwLock::new(HashMap::new())),
            event_manager,
        }
    }

    /// 启动事件监听
    pub async fn start(&self) {
        let filter = crate::event_manager::EventFilter::by_types(vec![
            EventType::LogPlayerConnected,
            EventType::LogPlayerDisconnected,
            EventType::LogJoinSucceeded,
        ]);

        let subscriber = self.event_manager.subscribe(filter).await;
        let sub_id = subscriber.id;

        tracing::info!("玩家追踪器启动，订阅 ID: {}", sub_id);

        let players = self.players.clone();
        let name_index = self.name_index.clone();
        let steam_index = self.steam_index.clone();
        let event_manager = self.event_manager.clone();

        tokio::spawn(async move {
            let mut channel = subscriber.channel;
            while let Some(event) = channel.recv().await {
                handle_player_event(&event, &players, &name_index, &steam_index, &event_manager).await;
            }
            tracing::info!("玩家追踪器事件监听停止");
        });
    }

    /// 从 RCON ServerStateReport 更新玩家数据
    pub async fn update_from_rcon(
        &self,
        rcon_players: &[crate::protocol::PlayerInfo],
        rcon_squads: &[crate::protocol::SquadInfo],
    ) {
        let mut players = self.players.write().await;
        let mut name_index = self.name_index.write().await;
        let mut steam_index = self.steam_index.write().await;

        let mut connected_ids: HashMap<String, bool> = HashMap::new();

        for p in rcon_players {
            let steam_id = p.steam_id.clone();
            if steam_id.is_empty() { continue; }

            connected_ids.insert(steam_id.clone(), true);

            let _squad_name = rcon_squads.iter()
                .find(|s| s.squad_id == p.squad_id.clone().unwrap_or_default())
                .map(|s| s.name.clone())
                .unwrap_or_default();

            let state = PlayerState {
                eos_id: String::new(),
                steam_id: steam_id.clone(),
                name: p.name.clone(),
                team_id: p.team_id.to_string(),
                squad_id: p.squad_id.clone().unwrap_or_default(),
                role: p.role.clone(),
                is_connected: true,
                last_updated: chrono::Utc::now(),
            };

            if !p.name.is_empty() {
                name_index.insert(p.name.clone(), steam_id.clone());
            }
            steam_index.insert(steam_id.clone(), steam_id.clone());

            players.insert(steam_id, state);
        }

        for (key, state) in players.iter_mut() {
            if !connected_ids.contains_key(key.as_str()) && state.is_connected {
                state.is_connected = false;
                state.last_updated = chrono::Utc::now();
            }
        }
    }

    /// 检查是否为误伤
    pub async fn is_teamkill(&self, attacker_steam: &str, victim_steam: &str) -> bool {
        if attacker_steam == victim_steam { return false; }
        let players = self.players.read().await;
        match (players.get(attacker_steam), players.get(victim_steam)) {
            (Some(a), Some(v)) => !a.team_id.is_empty() && !v.team_id.is_empty() && a.team_id == v.team_id,
            _ => false,
        }
    }

    /// 通过 Steam ID 查找玩家
    pub async fn get_player_by_steam_id(&self, steam_id: &str) -> Option<PlayerState> {
        self.players.read().await.get(steam_id).cloned()
    }

    /// 通过名称查找玩家
    pub async fn get_player_by_name(&self, name: &str) -> Option<PlayerState> {
        let name_index = self.name_index.read().await;
        let players = self.players.read().await;
        name_index.get(name).and_then(|key| players.get(key).cloned())
    }

    /// 获取所有在线玩家
    pub async fn get_connected_players(&self) -> Vec<PlayerState> {
        self.players.read().await.values().filter(|p| p.is_connected).cloned().collect()
    }

    /// 清理长时间断连的玩家
    pub async fn cleanup_disconnected(&self, max_age_secs: i64) {
        let now = chrono::Utc::now();
        let mut players = self.players.write().await;
        let mut name_index = self.name_index.write().await;
        let mut steam_index = self.steam_index.write().await;

        players.retain(|_key, state| {
            if !state.is_connected && (now - state.last_updated).num_seconds() > max_age_secs {
                name_index.remove(&state.name);
                steam_index.remove(&state.steam_id);
                false
            } else {
                true
            }
        });
    }
}

/// 处理玩家事件
async fn handle_player_event(
    event: &Event,
    players: &Arc<RwLock<HashMap<String, PlayerState>>>,
    name_index: &Arc<RwLock<HashMap<String, String>>>,
    steam_index: &Arc<RwLock<HashMap<String, String>>>,
    event_manager: &Arc<EventManager>,
) {
    match event.event_type {
        EventType::LogPlayerConnected => {
            let eos_id = event.data.get("eos_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let steam_id = event.data.get("steam_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let name = event.data.get("name")
                .or_else(|| event.data.get("player_controller"))
                .and_then(|v| v.as_str()).unwrap_or("").to_string();

            let key = if !steam_id.is_empty() { steam_id.clone() } else { eos_id.clone() };
            if key.is_empty() { return; }

            let state = PlayerState {
                eos_id: eos_id.clone(),
                steam_id: steam_id.clone(),
                name: name.clone(),
                team_id: String::new(),
                squad_id: String::new(),
                role: String::new(),
                is_connected: true,
                last_updated: chrono::Utc::now(),
            };

            {
                let mut players_w = players.write().await;
                let mut name_index_w = name_index.write().await;
                let mut steam_index_w = steam_index.write().await;

                if !name.is_empty() {
                    name_index_w.insert(name.clone(), key.clone());
                }
                if !steam_id.is_empty() {
                    steam_index_w.insert(steam_id.clone(), key.clone());
                }
                players_w.insert(key, state);
            }

            tracing::debug!("玩家连接: {} (steam: {}, eos: {})", name, steam_id, eos_id);

            event_manager.publish(Event::new(
                EventType::PlayerConnected,
                serde_json::json!({
                    "steam_id": steam_id,
                    "eos_id": eos_id,
                    "name": name,
                }),
                event.raw_log.clone(),
            ));
        }
        EventType::LogPlayerDisconnected => {
            let eos_id = event.data.get("eos_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let steam_id = event.data.get("steam_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let name = event.data.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();

            let key = if !steam_id.is_empty() { steam_id.clone() } else { eos_id.clone() };
            if key.is_empty() { return; }

            {
                let mut players_w = players.write().await;
                if let Some(state) = players_w.get_mut(&key) {
                    state.is_connected = false;
                    state.last_updated = chrono::Utc::now();
                }
            }

            tracing::debug!("玩家断连: {} (steam: {}, eos: {})", name, steam_id, eos_id);

            event_manager.publish(Event::new(
                EventType::PlayerDisconnected,
                serde_json::json!({
                    "steam_id": steam_id,
                    "eos_id": eos_id,
                    "name": name,
                }),
                event.raw_log.clone(),
            ));
        }
        EventType::LogJoinSucceeded => {
            let eos_id = event.data.get("eos_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let steam_id = event.data.get("steam_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let name = event.data.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();

            let key = if !steam_id.is_empty() { steam_id.clone() } else { eos_id.clone() };
            if key.is_empty() { return; }

            let mut players_w = players.write().await;
            if let Some(state) = players_w.get_mut(&key) {
                state.is_connected = true;
                state.last_updated = chrono::Utc::now();
            } else {
                let state = PlayerState {
                    eos_id: eos_id.clone(),
                    steam_id: steam_id.clone(),
                    name: name.clone(),
                    team_id: String::new(),
                    squad_id: String::new(),
                    role: String::new(),
                    is_connected: true,
                    last_updated: chrono::Utc::now(),
                };
                players_w.insert(key, state);
            }
        }
        _ => {}
    }
}