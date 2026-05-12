use axum::{Json, extract::{State, Path, Query}, http::StatusCode};
use serde::Deserialize;
use chrono::{DateTime, Utc, Duration};

use crate::api::AppState;

#[derive(Deserialize)]
pub struct TimeRangeQuery {
    pub start: Option<String>,
    pub end: Option<String>,
    pub days: Option<i64>,
}

#[derive(Deserialize)]
pub struct StatsQuery {
    #[serde(flatten)]
    pub time_range: TimeRangeQuery,
    pub limit: Option<usize>,
}

fn parse_time_range(query: &TimeRangeQuery) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    let end = Utc::now();
    let start = if let Some(days) = query.days {
        Some(end - Duration::days(days))
    } else if let Some(start_str) = &query.start {
        DateTime::parse_from_rfc3339(start_str)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    } else {
        Some(end - Duration::days(7))
    };

    let end = if let Some(end_str) = &query.end {
        DateTime::parse_from_rfc3339(end_str)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(end)
    } else {
        end
    };

    start.map(|s| (s, end))
}

// ClickHouse 查询结果结构体（clickhouse 0.15 要求 Row derive）

#[derive(clickhouse::Row, serde::Deserialize)]
struct CountRow {
    cnt: u64,
}

#[derive(clickhouse::Row, serde::Deserialize)]
struct WeaponStatRow {
    weapon: String,
    uses: u64,
    avg_damage: f32,
    teamkills: u64,
}

#[derive(clickhouse::Row, serde::Deserialize)]
struct PlayerKillsRow {
    attacker_name: String,
    attacker_steam64: String,
    kills: u64,
    teamkills: u64,
    avg_damage: f32,
}

#[derive(clickhouse::Row, serde::Deserialize)]
struct HourlyActiveRow {
    hour: DateTime<Utc>,
    active_players: u64,
}

#[derive(clickhouse::Row, serde::Deserialize)]
struct MatchStatRow {
    event_time: DateTime<Utc>,
    event_type: String,
    map_name: String,
    layer_name: String,
    team1_faction: String,
    team2_faction: String,
    winner_team: Option<i32>,
    team1_tickets: Option<i32>,
    team2_tickets: Option<i32>,
}

#[derive(clickhouse::Row, serde::Deserialize)]
struct TickRateAggRow {
    avg_tick: f32,
    min_tick: f32,
    max_tick: f32,
    p95_tick: f32,
}

#[derive(clickhouse::Row, serde::Deserialize)]
struct TickRateTrendRow {
    minute: DateTime<Utc>,
    avg_tick: f32,
}

/// 服务器综合统计
pub async fn server_stats(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Query(q): Query<StatsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let Some(pool) = &state.clickhouse_pool else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let Some((start, end)) = parse_time_range(&q.time_range) else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let client = pool.client();
    let db = pool.database.clone();

    let total_kills = tokio::spawn({
        let client = client.clone();
        let db1 = db.clone();
        async move {
            client
                .query(&format!(
                    "SELECT count() as cnt FROM {}.player_died_events WHERE server_id = ? AND event_time >= ? AND event_time <= ?",
                    db1
                ))
                .bind(server_id)
                .bind(start)
                .bind(end)
                .fetch_one::<CountRow>()
                .await
                .map(|r| r.cnt)
                .ok()
        }
    });

    let total_teamkills = tokio::spawn({
        let client = client.clone();
        let db2 = db.clone();
        async move {
            client
                .query(&format!(
                    "SELECT count() as cnt FROM {}.player_died_events WHERE server_id = ? AND event_time >= ? AND event_time <= ? AND teamkill = 1",
                    db2
                ))
                .bind(server_id)
                .bind(start)
                .bind(end)
                .fetch_one::<CountRow>()
                .await
                .map(|r| r.cnt)
                .ok()
        }
    });

    let unique_players = tokio::spawn({
        let client = client.clone();
        let db3 = db.clone();
        async move {
            client
                .query(&format!(
                    "SELECT uniqExact(attacker_steam64) + uniqExact(victim_steam64) as cnt FROM {}.player_died_events WHERE server_id = ? AND event_time >= ? AND event_time <= ?",
                    db3
                ))
                .bind(server_id)
                .bind(start)
                .bind(end)
                .fetch_one::<CountRow>()
                .await
                .map(|r| r.cnt)
                .ok()
        }
    });

    let total_chats = tokio::spawn({
        let client = client.clone();
        let db4 = db.clone();
        async move {
            client
                .query(&format!(
                    "SELECT count() as cnt FROM {}.player_chat_messages WHERE server_id = ? AND sent_at >= ? AND sent_at <= ?",
                    db4
                ))
                .bind(server_id)
                .bind(start)
                .bind(end)
                .fetch_one::<CountRow>()
                .await
                .map(|r| r.cnt)
                .ok()
        }
    });

    let (total_kills, total_teamkills, unique_players, total_chats) = tokio::join!(
        total_kills,
        total_teamkills,
        unique_players,
        total_chats
    );

    Ok(Json(serde_json::json!({
        "server_id": server_id,
        "period": {
            "start": start,
            "end": end
        },
        "stats": {
            "total_kills": total_kills.ok().flatten().unwrap_or(0),
            "total_teamkills": total_teamkills.ok().flatten().unwrap_or(0),
            "unique_players": unique_players.ok().flatten().unwrap_or(0),
            "total_chats": total_chats.ok().flatten().unwrap_or(0)
        }
    })))
}

/// 武器使用统计
pub async fn weapon_stats(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Query(q): Query<StatsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let Some(pool) = &state.clickhouse_pool else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let Some((start, end)) = parse_time_range(&q.time_range) else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let client = pool.client();
    let db = pool.database.clone();
    let limit = q.limit.unwrap_or(20);

    let result = client
        .query(&format!(
            "SELECT
                weapon,
                count() as uses,
                avg(damage) as avg_damage,
                countIf(teamkill=1) as teamkills
            FROM {}.player_died_events
            WHERE server_id = ? AND event_time >= ? AND event_time <= ?
            GROUP BY weapon
            ORDER BY uses DESC
            LIMIT ?",
            db
        ))
        .bind(server_id)
        .bind(start)
        .bind(end)
        .bind(limit)
        .fetch_all::<WeaponStatRow>()
        .await;

    match result {
        Ok(rows) => {
            let data: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|r| {
                    serde_json::json!({
                        "weapon": r.weapon,
                        "uses": r.uses,
                        "avg_damage": r.avg_damage,
                        "teamkills": r.teamkills
                    })
                })
                .collect();
            Ok(Json(serde_json::json!({ "data": data })))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 玩家击杀排行榜
pub async fn player_kills_leaderboard(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Query(q): Query<StatsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let Some(pool) = &state.clickhouse_pool else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let Some((start, end)) = parse_time_range(&q.time_range) else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let client = pool.client();
    let db = pool.database.clone();
    let limit = q.limit.unwrap_or(50);

    let result = client
        .query(&format!(
            "SELECT
                attacker_name,
                attacker_steam64,
                count() as kills,
                countIf(teamkill=1) as teamkills,
                avg(damage) as avg_damage
            FROM {}.player_died_events
            WHERE server_id = ? AND event_time >= ? AND event_time <= ?
            GROUP BY attacker_name, attacker_steam64
            ORDER BY kills DESC
            LIMIT ?",
            db
        ))
        .bind(server_id)
        .bind(start)
        .bind(end)
        .bind(limit)
        .fetch_all::<PlayerKillsRow>()
        .await;

    match result {
        Ok(rows) => {
            let data: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|r| {
                    serde_json::json!({
                        "player_name": r.attacker_name,
                        "steam64": r.attacker_steam64,
                        "kills": r.kills,
                        "teamkills": r.teamkills,
                        "avg_damage": r.avg_damage
                    })
                })
                .collect();
            Ok(Json(serde_json::json!({ "data": data })))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 在线玩家趋势
pub async fn player_activity_trend(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Query(q): Query<TimeRangeQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let Some(pool) = &state.clickhouse_pool else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let Some((start, end)) = parse_time_range(&q) else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let client = pool.client();
    let db = pool.database.clone();

    let result = client
        .query(&format!(
            "SELECT
                toStartOfHour(event_time) as hour,
                uniqExact(attacker_steam64) as active_players
            FROM {}.player_died_events
            WHERE server_id = ? AND event_time >= ? AND event_time <= ?
            GROUP BY hour
            ORDER BY hour ASC",
            db
        ))
        .bind(server_id)
        .bind(start)
        .bind(end)
        .fetch_all::<HourlyActiveRow>()
        .await;

    match result {
        Ok(rows) => {
            let data: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|r| {
                    serde_json::json!({
                        "hour": r.hour,
                        "active_players": r.active_players
                    })
                })
                .collect();
            Ok(Json(serde_json::json!({ "data": data })))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 比赛统计
pub async fn match_stats(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Query(q): Query<StatsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let Some(pool) = &state.clickhouse_pool else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let Some((start, end)) = parse_time_range(&q.time_range) else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let client = pool.client();
    let db = pool.database.clone();
    let limit = q.limit.unwrap_or(50);

    let result = client
        .query(&format!(
            "SELECT
                event_time,
                event_type,
                map_name,
                layer_name,
                team1_faction,
                team2_faction,
                winner_team,
                team1_tickets,
                team2_tickets
            FROM {}.match_events
            WHERE server_id = ? AND event_time >= ? AND event_time <= ?
            ORDER BY event_time DESC
            LIMIT ?",
            db
        ))
        .bind(server_id)
        .bind(start)
        .bind(end)
        .bind(limit)
        .fetch_all::<MatchStatRow>()
        .await;

    match result {
        Ok(rows) => {
            let data: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|r| {
                    serde_json::json!({
                        "event_time": r.event_time,
                        "event_type": r.event_type,
                        "map_name": r.map_name,
                        "layer_name": r.layer_name,
                        "team1_faction": r.team1_faction,
                        "team2_faction": r.team2_faction,
                        "winner_team": r.winner_team,
                        "team1_tickets": r.team1_tickets,
                        "team2_tickets": r.team2_tickets
                    })
                })
                .collect();
            Ok(Json(serde_json::json!({ "data": data })))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Tick 率统计
pub async fn tick_rate_stats(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Query(q): Query<StatsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let Some(pool) = &state.clickhouse_pool else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let Some((start, end)) = parse_time_range(&q.time_range) else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let client = pool.client();
    let db = pool.database.clone();

    let stats = client
        .query(&format!(
            "SELECT
                avg(tick_rate) as avg_tick,
                min(tick_rate) as min_tick,
                max(tick_rate) as max_tick,
                quantile(0.95)(tick_rate) as p95_tick
            FROM {}.tick_rate_events
            WHERE server_id = ? AND event_time >= ? AND event_time <= ?",
            db
        ))
        .bind(server_id)
        .bind(start)
        .bind(end)
        .fetch_one::<TickRateAggRow>()
        .await;

    let trend = client
        .query(&format!(
            "SELECT
                toStartOfMinute(event_time) as minute,
                avg(tick_rate) as avg_tick
            FROM {}.tick_rate_events
            WHERE server_id = ? AND event_time >= ? AND event_time <= ?
            GROUP BY minute
            ORDER BY minute ASC
            LIMIT 1440",
            db
        ))
        .bind(server_id)
        .bind(start)
        .bind(end)
        .fetch_all::<TickRateTrendRow>()
        .await;

    match (stats, trend) {
        (Ok(s), Ok(trend_rows)) => {
            let trend_data: Vec<serde_json::Value> = trend_rows
                .into_iter()
                .map(|r| {
                    serde_json::json!({
                        "minute": r.minute,
                        "tick_rate": r.avg_tick
                    })
                })
                .collect();
            Ok(Json(serde_json::json!({
                "stats": {
                    "avg": s.avg_tick,
                    "min": s.min_tick,
                    "max": s.max_tick,
                    "p95": s.p95_tick
                },
                "trend": trend_data
            })))
        }
        _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Check ClickHouse health
pub async fn health_check(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some(pool) = &state.clickhouse_pool {
        let healthy = pool.health_check().await;
        Ok(Json(serde_json::json!({
            "clickhouse": if healthy { "healthy" } else { "unhealthy" },
            "database": pool.database
        })))
    } else {
        Ok(Json(serde_json::json!({
            "clickhouse": "disabled"
        })))
    }
}
