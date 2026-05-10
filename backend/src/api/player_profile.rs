use axum::{Json, extract::{State, Path}, http::StatusCode};
use crate::api::AppState;

/// GET /api/v1/player-profile/{steam64}
/// 聚合玩家完整档案数据
pub async fn get_profile(
    State(state): State<AppState>,
    Path(steam64): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let pool = &state.pool;

    // 1. 跨服务器玩家信息聚合
    let player_rows = sqlx::query_as::<_, (String, String, String, String, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
        "SELECT player_name, steam64, eos_id, ip, first_seen, last_seen FROM player_info WHERE steam64 = $1 ORDER BY last_seen DESC"
    ).bind(&steam64).fetch_all(pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if player_rows.is_empty() {
        return Ok(Json(serde_json::json!({ "error": "未找到该玩家" })));
    }

    let player_name = player_rows.first().map(|r| r.0.clone()).unwrap_or_default();
    let eos_id = player_rows.first().map(|r| r.2.clone()).unwrap_or_default();
    let last_known_ip = player_rows.first().map(|r| r.3.clone()).unwrap_or_default();
    let first_seen = player_rows.iter().map(|r| r.4).min();
    let last_seen = player_rows.iter().map(|r| r.5).max();

    // 名称历史（按名称分组统计出现次数）
    let mut name_map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (name, _, _, _, _, _) in &player_rows {
        *name_map.entry(name.clone()).or_insert(0) += 1;
    }
    let name_history: Vec<serde_json::Value> = name_map.into_iter().map(|(name, count)| {
        serde_json::json!({ "name": name, "session_count": count })
    }).collect();

    // 最近服务器
    let recent_servers: Vec<serde_json::Value> = player_rows.iter().take(10).map(|(_, _, _, _, _, ls)| {
        serde_json::json!({ "last_seen": ls })
    }).collect();

    // 2. 聊天历史（最近100条，跨所有服务器）
    let chat_rows = sqlx::query_as::<_, (String, String, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT player_name, message, channel, logged_at FROM chat_messages WHERE steam64 = $1 ORDER BY logged_at DESC LIMIT 100"
    ).bind(&steam64).fetch_all(pool).await.unwrap_or_default();

    let chat_history: Vec<serde_json::Value> = chat_rows.into_iter().map(|(name, msg, ch, ts)| {
        serde_json::json!({ "player_name": name, "message": msg, "channel": ch, "logged_at": ts })
    }).collect();

    // 3. 击杀统计
    let (total_kills,) = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM kill_events WHERE attacker_steam64 = $1"
    ).bind(&steam64).fetch_one(pool).await.unwrap_or((0,));

    let (total_deaths,) = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM kill_events WHERE victim_steam64 = $1"
    ).bind(&steam64).fetch_one(pool).await.unwrap_or((0,));

    let (total_teamkills,) = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM kill_events WHERE attacker_steam64 = $1 AND is_teamkill = true"
    ).bind(&steam64).fetch_one(pool).await.unwrap_or((0,));

    // 4. 战斗历史（最近50条）
    let combat_rows = sqlx::query_as::<_, (String, String, String, String, f64, String, String, bool, chrono::DateTime<chrono::Utc>)>(
        "SELECT attacker_name, victim_name, attacker_steam64, victim_steam64, damage, weapon, event_type, is_teamkill, logged_at FROM kill_events WHERE attacker_steam64 = $1 OR victim_steam64 = $1 ORDER BY logged_at DESC LIMIT 50"
    ).bind(&steam64).fetch_all(pool).await.unwrap_or_default();

    let combat_history: Vec<serde_json::Value> = combat_rows.into_iter().map(|(an, vn, as64, vs64, dmg, wp, et, itk, ts)| {
        let is_attacker = as64 == steam64;
        let event_type = if is_attacker {
            if itk { "teamkill" } else { "kill" }
        } else {
            "death"
        };
        let other_name = if is_attacker { vn } else { an };
        let other_steam64 = if is_attacker { vs64 } else { as64 };
        serde_json::json!({
            "event_time": ts, "event_type": event_type, "weapon": wp,
            "damage": dmg, "teamkill": itk,
            "other_name": other_name, "other_steam64": other_steam64,
            "is_attacker": is_attacker
        })
    }).collect();

    // 5. 聊天违规记录
    let violation_rows = sqlx::query_as::<_, (String, String, String, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT player_name, message, category, action_taken, logged_at FROM chat_violations WHERE steam_id = $1 ORDER BY logged_at DESC LIMIT 50"
    ).bind(&steam64).fetch_all(pool).await.unwrap_or_default();

    let violations: Vec<serde_json::Value> = violation_rows.into_iter().map(|(name, msg, cat, action, ts)| {
        serde_json::json!({ "player_name": name, "message": msg, "category": cat, "action_taken": action, "logged_at": ts })
    }).collect();

    let violation_summary = serde_json::json!({
        "total_warns": violations.iter().filter(|v| v["action_taken"] == "WARN").count(),
        "total_kicks": violations.iter().filter(|v| v["action_taken"] == "KICK").count(),
        "total_bans": violations.iter().filter(|v| v["action_taken"] == "BAN").count(),
    });

    // 6. 身份归并信息
    let identity = sqlx::query_as::<_, (String, String, Vec<String>, Vec<String>, Vec<String>, i32, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>)>(
        "SELECT canonical_id, primary_name, all_steam_ids, all_eos_ids, all_names, total_sessions, first_seen, last_seen FROM player_identities WHERE primary_steam_id = $1"
    ).bind(&steam64).fetch_optional(pool).await.unwrap_or(None);

    let identity_info = identity.map(|(cid, pname, steam_ids, eos_ids, names, sessions, fs, ls)| {
        serde_json::json!({
            "canonical_id": cid, "primary_name": pname,
            "all_steam_ids": steam_ids, "all_eos_ids": eos_ids,
            "all_names": names, "total_sessions": sessions,
            "first_seen": fs, "last_seen": ls,
            "identity_status": "resolved"
        })
    });

    // 7. TK 统计（最近7天）
    let (recent_teamkills,) = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM kill_events WHERE attacker_steam64 = $1 AND is_teamkill = true AND logged_at > NOW() - INTERVAL '7 days'"
    ).bind(&steam64).fetch_one(pool).await.unwrap_or((0,));

    // 武器统计
    let weapon_rows = sqlx::query_as::<_, (String, i64, i64)>(
        "SELECT weapon, COUNT(*) as kills, SUM(CASE WHEN is_teamkill THEN 1 ELSE 0 END) as teamkills FROM kill_events WHERE attacker_steam64 = $1 GROUP BY weapon ORDER BY kills DESC LIMIT 20"
    ).bind(&steam64).fetch_all(pool).await.unwrap_or_default();

    let weapon_stats: Vec<serde_json::Value> = weapon_rows.into_iter().map(|(w, k, tk)| {
        serde_json::json!({ "weapon": w, "kills": k, "teamkills": tk })
    }).collect();

    // 组装响应
    let profile = serde_json::json!({
        "steam_id": steam64,
        "eos_id": eos_id,
        "player_name": player_name,
        "last_known_ip": last_known_ip,
        "can_view_ip": true,
        "first_seen": first_seen,
        "last_seen": last_seen,
        "total_sessions": player_rows.len(),
        "total_play_time": 0, // 当前数据库没有 play_time 字段

        "name_history": name_history,
        "recent_servers": recent_servers,

        "statistics": {
            "kills": total_kills,
            "deaths": total_deaths,
            "teamkills": total_teamkills,
            "revives": 0,
            "times_revived": 0,
            "damage_dealt": 0,
            "damage_taken": 0,
            "kd_ratio": if total_deaths > 0 { total_kills as f64 / total_deaths as f64 } else { total_kills as f64 }
        },
        "current_match_statistics": {
            "kills": 0, "deaths": 0, "teamkills": 0,
            "revives": 0, "times_revived": 0,
            "damage_dealt": 0, "damage_taken": 0, "kd_ratio": 0.0
        },

        "chat_history": chat_history,
        "combat_history": combat_history,
        "violations": violations,
        "violation_summary": violation_summary,
        "teamkill_metrics": {
            "total_teamkills": total_teamkills,
            "teamkills_per_session": if player_rows.len() > 0 { total_teamkills as f64 / player_rows.len() as f64 } else { 0.0 },
            "teamkill_ratio": if total_kills > 0 { total_teamkills as f64 / total_kills as f64 } else { 0.0 },
            "recent_teamkills": recent_teamkills,
            "total_team_wounds": 0,
            "total_team_damage": 0,
            "recent_team_wounds": 0,
            "recent_team_damage": 0
        },

        "weapon_stats": weapon_stats,
        "active_bans": [],
        "risk_indicators": [],
        "identity": identity_info
    });

    Ok(Json(serde_json::json!({ "data": { "player": profile } })))
}
