use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 聊天消息事件
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct ChatMessageEvent {
    pub event_time: DateTime<Utc>,
    pub server_id: i32,
    pub player_name: String,
    pub player_steam64: String,
    pub chat_type: String,
    pub message: String,
}

/// 玩家伤害事件
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct PlayerDamagedEvent {
    pub event_time: DateTime<Utc>,
    pub server_id: i32,
    pub attacker_name: String,
    pub attacker_steam64: String,
    pub victim_name: String,
    pub victim_steam64: String,
    pub weapon: String,
    pub damage: f32,
    pub teamkill: u8,
}

/// 玩家死亡事件
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct PlayerDiedEvent {
    pub event_time: DateTime<Utc>,
    pub server_id: i32,
    pub attacker_name: String,
    pub attacker_steam64: String,
    pub victim_name: String,
    pub victim_steam64: String,
    pub weapon: String,
    pub damage: f32,
    pub teamkill: u8,
}

/// 玩家连接事件
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct PlayerConnectedEvent {
    pub event_time: DateTime<Utc>,
    pub server_id: i32,
    pub player_name: String,
    pub player_steam64: String,
    pub action: String,
}

/// 可部署物事件
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct DeployableEvent {
    pub event_time: DateTime<Utc>,
    pub server_id: i32,
    pub player_name: String,
    pub player_steam64: String,
    pub deployable: String,
    pub action: String,
}

/// Tick 率事件
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct TickRateEvent {
    pub event_time: DateTime<Utc>,
    pub server_id: i32,
    pub tick_rate: f32,
}

/// 比赛事件
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct MatchEvent {
    pub event_time: DateTime<Utc>,
    pub server_id: i32,
    pub event_type: String,
    pub map_name: String,
    pub layer_name: String,
    pub team1_faction: String,
    pub team2_faction: String,
    pub winner_team: Option<i32>,
    pub team1_tickets: Option<i32>,
    pub team2_tickets: Option<i32>,
}

/// 载具事件
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct VehicleEvent {
    pub event_time: DateTime<Utc>,
    pub server_id: i32,
    pub vehicle_name: String,
    pub action: String,
    pub player_name: String,
    pub player_steam64: String,
}

/// 载具伤害事件
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct VehicleDamageEvent {
    pub event_time: DateTime<Utc>,
    pub server_id: i32,
    pub attacker_name: String,
    pub attacker_steam64: String,
    pub vehicle_name: String,
    pub damage: f32,
    pub weapon: String,
}

/// 飞行事件
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct FlyEvent {
    pub event_time: DateTime<Utc>,
    pub server_id: i32,
    pub player_name: String,
    pub player_steam64: String,
    pub location: String,
    pub altitude: f32,
}

/// 建表语句
pub const TABLE_DEFINITIONS: &[&str] = &[
    // 聊天消息
    r#"CREATE TABLE IF NOT EXISTS player_chat_messages (
        event_time DateTime,
        server_id Int32,
        player_name String,
        player_steam64 String,
        chat_type LowCardinality(String),
        message String
    ) ENGINE = MergeTree()
    PARTITION BY toYYYYMM(event_time)
    ORDER BY (server_id, event_time)"#,

    // 玩家伤害
    r#"CREATE TABLE IF NOT EXISTS player_damaged_events (
        event_time DateTime,
        server_id Int32,
        attacker_name String,
        attacker_steam64 String,
        victim_name String,
        victim_steam64 String,
        weapon LowCardinality(String),
        damage Float32,
        teamkill UInt8
    ) ENGINE = MergeTree()
    PARTITION BY toYYYYMM(event_time)
    ORDER BY (server_id, event_time, victim_steam64)"#,

    // 玩家死亡
    r#"CREATE TABLE IF NOT EXISTS player_died_events (
        event_time DateTime,
        server_id Int32,
        attacker_name String,
        attacker_steam64 String,
        victim_name String,
        victim_steam64 String,
        weapon LowCardinality(String),
        damage Float32,
        teamkill UInt8
    ) ENGINE = MergeTree()
    PARTITION BY toYYYYMM(event_time)
    ORDER BY (server_id, event_time, victim_steam64)"#,

    // 玩家连接
    r#"CREATE TABLE IF NOT EXISTS player_connected_events (
        event_time DateTime,
        server_id Int32,
        player_name String,
        player_steam64 String,
        action LowCardinality(String)
    ) ENGINE = MergeTree()
    PARTITION BY toYYYYMM(event_time)
    ORDER BY (server_id, event_time, player_steam64)"#,

    // 可部署物
    r#"CREATE TABLE IF NOT EXISTS deployable_events (
        event_time DateTime,
        server_id Int32,
        player_name String,
        player_steam64 String,
        deployable LowCardinality(String),
        action LowCardinality(String)
    ) ENGINE = MergeTree()
    PARTITION BY toYYYYMM(event_time)
    ORDER BY (server_id, event_time)"#,

    // Tick 率
    r#"CREATE TABLE IF NOT EXISTS tick_rate_events (
        event_time DateTime,
        server_id Int32,
        tick_rate Float32
    ) ENGINE = MergeTree()
    PARTITION BY toYYYYMM(event_time)
    ORDER BY (server_id, event_time)"#,

    // 比赛
    r#"CREATE TABLE IF NOT EXISTS match_events (
        event_time DateTime,
        server_id Int32,
        event_type LowCardinality(String),
        map_name LowCardinality(String),
        layer_name String,
        team1_faction LowCardinality(String),
        team2_faction LowCardinality(String),
        winner_team Nullable(Int32),
        team1_tickets Nullable(Int32),
        team2_tickets Nullable(Int32)
    ) ENGINE = MergeTree()
    PARTITION BY toYYYYMM(event_time)
    ORDER BY (server_id, event_time)"#,

    // 载具
    r#"CREATE TABLE IF NOT EXISTS vehicle_events (
        event_time DateTime,
        server_id Int32,
        vehicle_name LowCardinality(String),
        action LowCardinality(String),
        player_name String,
        player_steam64 String
    ) ENGINE = MergeTree()
    PARTITION BY toYYYYMM(event_time)
    ORDER BY (server_id, event_time)"#,

    // 载具伤害
    r#"CREATE TABLE IF NOT EXISTS vehicle_damage_events (
        event_time DateTime,
        server_id Int32,
        attacker_name String,
        attacker_steam64 String,
        vehicle_name LowCardinality(String),
        damage Float32,
        weapon LowCardinality(String)
    ) ENGINE = MergeTree()
    PARTITION BY toYYYYMM(event_time)
    ORDER BY (server_id, event_time)"#,

    // 飞行
    r#"CREATE TABLE IF NOT EXISTS fly_events (
        event_time DateTime,
        server_id Int32,
        player_name String,
        player_steam64 String,
        location String,
        altitude Float32
    ) ENGINE = MergeTree()
    PARTITION BY toYYYYMM(event_time)
    ORDER BY (server_id, event_time)"#,
];

/// TTL 语句（数据保留策略）
pub const TTL_STATEMENTS: &[&str] = &[
    "ALTER TABLE player_chat_messages MODIFY TTL event_time + INTERVAL 90 DAY",
    "ALTER TABLE player_damaged_events MODIFY TTL event_time + INTERVAL 90 DAY",
    "ALTER TABLE player_died_events MODIFY TTL event_time + INTERVAL 90 DAY",
    "ALTER TABLE player_connected_events MODIFY TTL event_time + INTERVAL 90 DAY",
    "ALTER TABLE deployable_events MODIFY TTL event_time + INTERVAL 90 DAY",
    "ALTER TABLE tick_rate_events MODIFY TTL event_time + INTERVAL 30 DAY",
    "ALTER TABLE match_events MODIFY TTL event_time + INTERVAL 365 DAY",
    "ALTER TABLE vehicle_events MODIFY TTL event_time + INTERVAL 90 DAY",
    "ALTER TABLE vehicle_damage_events MODIFY TTL event_time + INTERVAL 90 DAY",
    "ALTER TABLE fly_events MODIFY TTL event_time + INTERVAL 90 DAY",
];

/// 获取所有表的创建语句
pub fn get_all_create_statements() -> Vec<String> {
    TABLE_DEFINITIONS.iter().map(|s| s.to_string()).collect()
}
