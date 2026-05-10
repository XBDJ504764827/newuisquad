use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use serde::Serialize;

// ═══ EventType Enum ═══

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum EventType {
    // RCON events
    RconChatMessage,
    RconPlayerWarned,
    RconPlayerKicked,
    RconPlayerBanned,
    RconSquadCreated,
    // Log events
    LogPlayerConnected,
    LogPlayerDisconnected,
    LogPlayerDied,
    LogPlayerWounded,
    LogPlayerRevived,
    LogPlayerDamaged,
    LogAdminBroadcast,
    LogGameEventUnified,
    // Player tracker events
    PlayerListUpdated,
    PlayerConnected,
    PlayerDisconnected,
    PlayerTeamChanged,
    PlayerSquadChanged,
    SquadCreated,
    SquadDisbanded,
    EnhancedTeamkill,
    // Plugin/system
    PluginCustom,
    PluginLog,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::RconChatMessage => "RCON_CHAT_MESSAGE",
            EventType::RconPlayerWarned => "RCON_PLAYER_WARNED",
            EventType::RconPlayerKicked => "RCON_PLAYER_KICKED",
            EventType::RconPlayerBanned => "RCON_PLAYER_BANNED",
            EventType::RconSquadCreated => "RCON_SQUAD_CREATED",
            EventType::LogPlayerConnected => "LOG_PLAYER_CONNECTED",
            EventType::LogPlayerDisconnected => "LOG_PLAYER_DISCONNECTED",
            EventType::LogPlayerDied => "LOG_PLAYER_DIED",
            EventType::LogPlayerWounded => "LOG_PLAYER_WOUNDED",
            EventType::LogPlayerRevived => "LOG_PLAYER_REVIVED",
            EventType::LogPlayerDamaged => "LOG_PLAYER_DAMAGED",
            EventType::LogAdminBroadcast => "LOG_ADMIN_BROADCAST",
            EventType::LogGameEventUnified => "LOG_GAME_EVENT_UNIFIED",
            EventType::PlayerListUpdated => "PLAYER_LIST_UPDATED",
            EventType::PlayerConnected => "PLAYER_CONNECTED",
            EventType::PlayerDisconnected => "PLAYER_DISCONNECTED",
            EventType::PlayerTeamChanged => "PLAYER_TEAM_CHANGED",
            EventType::PlayerSquadChanged => "PLAYER_SQUAD_CHANGED",
            EventType::SquadCreated => "SQUAD_CREATED",
            EventType::SquadDisbanded => "SQUAD_DISBANDED",
            EventType::EnhancedTeamkill => "ENHANCED_TEAMKILL",
            EventType::PluginCustom => "PLUGIN_CUSTOM",
            EventType::PluginLog => "PLUGIN_LOG",
        }
    }
}

// ═══ Event Data Structs ═══

#[derive(Debug, Clone, Serialize)]
pub struct PlayerConnectedData {
    pub player_name: String,
    pub steam_id: String,
    pub eos_id: String,
    pub ip_address: String,
    pub player_controller: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerDisconnectedData {
    pub player_name: String,
    pub steam_id: String,
    pub eos_id: String,
    pub team_id: i32,
    pub squad_id: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerListUpdatedData {
    pub player_count: usize,
    pub team_count: usize,
    pub squad_count: usize,
    pub map_name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerDiedData {
    pub attacker_name: String,
    pub attacker_steam_id: String,
    pub attacker_eos_id: String,
    pub attacker_team_id: i32,
    pub victim_name: String,
    pub victim_steam_id: String,
    pub victim_eos_id: String,
    pub victim_team_id: i32,
    pub damage: f64,
    pub weapon: String,
    pub is_teamkill: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatMessageData {
    pub player_name: String,
    pub steam_id: String,
    pub eos_id: String,
    pub message: String,
    pub chat_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TeamkillData {
    pub attacker_name: String,
    pub attacker_steam_id: String,
    pub victim_name: String,
    pub victim_steam_id: String,
    pub damage: f64,
    pub weapon: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerWarnedData {
    pub player_name: String,
    pub steam_id: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerKickedData {
    pub player_name: String,
    pub steam_id: String,
    pub player_id: i32,
    pub reason: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerBannedData {
    pub player_name: String,
    pub steam_id: String,
    pub duration: String,
    pub reason: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Unified game event (sent from PlayerTracker and other services)
#[derive(Debug, Clone, Serialize)]
pub struct GameEvent {
    pub server_id: i32,
    pub event_type: EventType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_connected: Option<PlayerConnectedData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_disconnected: Option<PlayerDisconnectedData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_list_updated: Option<PlayerListUpdatedData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_died: Option<PlayerDiedData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat_message: Option<ChatMessageData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub teamkill: Option<TeamkillData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_warned: Option<PlayerWarnedData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_kicked: Option<PlayerKickedData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_banned: Option<PlayerBannedData>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl GameEvent {
    pub fn new(server_id: i32, event_type: EventType) -> Self {
        Self {
            server_id, event_type,
            player_connected: None,
            player_disconnected: None,
            player_list_updated: None,
            player_died: None,
            chat_message: None,
            teamkill: None,
            player_warned: None,
            player_kicked: None,
            player_banned: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn player_connected(server_id: i32, data: PlayerConnectedData) -> Self {
        Self { player_connected: Some(data), ..Self::new(server_id, EventType::PlayerConnected) }
    }

    pub fn player_died(server_id: i32, data: PlayerDiedData) -> Self {
        Self { player_died: Some(data), ..Self::new(server_id, EventType::LogPlayerDied) }
    }

    pub fn teamkill(server_id: i32, data: TeamkillData) -> Self {
        Self { teamkill: Some(data), ..Self::new(server_id, EventType::EnhancedTeamkill) }
    }
}

// ═══ Event Manager ═══

pub struct EventManager {
    tx: broadcast::Sender<GameEvent>,
}

impl EventManager {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    /// Publish an event (non-blocking — drops if channel full)
    pub fn publish(&self, event: GameEvent) {
        if self.tx.receiver_count() > 0 {
            let _ = self.tx.send(event);
        }
    }

    /// Subscribe to all events
    pub fn subscribe(&self) -> broadcast::Receiver<GameEvent> {
        self.tx.subscribe()
    }

    /// Publish player connected
    pub fn player_connected(&self, server_id: i32, name: &str, steam_id: &str, eos_id: &str) {
        self.publish(GameEvent::player_connected(server_id, PlayerConnectedData {
            player_name: name.to_string(),
            steam_id: steam_id.to_string(),
            eos_id: eos_id.to_string(),
            ip_address: String::new(),
            player_controller: String::new(),
            timestamp: chrono::Utc::now(),
        }));
    }

    /// Publish player died / teamkill
    pub fn player_died(&self, server_id: i32, attacker_name: &str, attacker_steam: &str,
        victim_name: &str, victim_steam: &str, attacker_team: i32, victim_team: i32,
        damage: f64, weapon: &str, is_teamkill: bool)
    {
        self.publish(GameEvent::player_died(server_id, PlayerDiedData {
            attacker_name: attacker_name.to_string(),
            attacker_steam_id: attacker_steam.to_string(),
            attacker_eos_id: String::new(),
            attacker_team_id: attacker_team,
            victim_name: victim_name.to_string(),
            victim_steam_id: victim_steam.to_string(),
            victim_eos_id: String::new(),
            victim_team_id: victim_team,
            damage, weapon: weapon.to_string(), is_teamkill,
            timestamp: chrono::Utc::now(),
        }));
    }
}
