use axum::{Json, extract::{State, Path, Query}, http::StatusCode};
use serde::Deserialize;
use crate::api::AppState;

#[derive(Deserialize, Default)]
pub struct PageQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    #[serde(default)]
    pub steam64: Option<String>,
}

pub async fn fly_events(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Query(q): Query<PageQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(50).min(200);
    let offset = (page - 1) * per_page;

    let (total,) = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM fly_events WHERE server_id=$1"
    ).bind(server_id).fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let items = sqlx::query_as::<_, (i32, i32, String, String, String, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT id, server_id, player_name, eos_id, steam64, event_type, logged_at FROM fly_events WHERE server_id=$1 ORDER BY logged_at DESC LIMIT $2 OFFSET $3"
    ).bind(server_id).bind(per_page).bind(offset).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let data: Vec<serde_json::Value> = items.into_iter().map(|(id, sid, name, eos, steam, evt, ts)| {
        serde_json::json!({ "id": id, "server_id": sid, "player_name": name, "eos_id": eos, "steam64": steam, "event_type": evt, "logged_at": ts })
    }).collect();

    Ok(Json(serde_json::json!({ "data": data, "total": total, "page": page, "per_page": per_page })))
}

pub async fn kill_events(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Query(q): Query<PageQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(50).min(200);
    let offset = (page - 1) * per_page;
    let steam64 = q.steam64.as_deref().filter(|sid| !sid.is_empty());

    let total = if let Some(sid) = steam64 {
        sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM kill_events WHERE server_id=$1 AND (attacker_steam64=$2 OR victim_steam64=$2)")
            .bind(server_id).bind(sid).fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.0
    } else {
        sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM kill_events WHERE server_id=$1")
            .bind(server_id).fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.0
    };

    let items = if let Some(sid) = steam64 {
        sqlx::query_as::<_, (i32, i32, String, String, String, String, String, String, f64, String, String, bool, bool, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, server_id, attacker_name, attacker_eos, attacker_steam64, victim_name, victim_eos, victim_steam64, damage, weapon, event_type, is_kill, is_teamkill, logged_at FROM kill_events WHERE server_id=$1 AND (attacker_steam64=$2 OR victim_steam64=$2) ORDER BY logged_at DESC LIMIT $3 OFFSET $4"
        ).bind(server_id).bind(sid).bind(per_page).bind(offset).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        sqlx::query_as::<_, (i32, i32, String, String, String, String, String, String, f64, String, String, bool, bool, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, server_id, attacker_name, attacker_eos, attacker_steam64, victim_name, victim_eos, victim_steam64, damage, weapon, event_type, is_kill, is_teamkill, logged_at FROM kill_events WHERE server_id=$1 ORDER BY logged_at DESC LIMIT $2 OFFSET $3"
        ).bind(server_id).bind(per_page).bind(offset).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    let data: Vec<serde_json::Value> = items.into_iter().map(|(id, sid, an, ae, as64, vn, ve, vs64, dmg, wp, et, ik, itk, ts)| {
        serde_json::json!({ "id": id, "server_id": sid, "attacker_name": an, "attacker_eos": ae, "attacker_steam64": as64, "victim_name": vn, "victim_eos": ve, "victim_steam64": vs64, "damage": dmg, "weapon": wp, "event_type": et, "is_kill": ik, "is_teamkill": itk, "logged_at": ts })
    }).collect();

    Ok(Json(serde_json::json!({ "data": data, "total": total, "page": page, "per_page": per_page })))
}

pub async fn match_events(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Query(q): Query<PageQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(20).min(200);
    let offset = (page - 1) * per_page;

    let (total,) = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM match_info WHERE server_id=$1"
    ).bind(server_id).fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let items = sqlx::query_as::<_, (i32, i32, String, String, String, String, Option<i32>, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT id, server_id, map_name, layer_name, team1_faction, team2_faction, winner_team, event_type, logged_at FROM match_info WHERE server_id=$1 ORDER BY logged_at DESC LIMIT $2 OFFSET $3"
    ).bind(server_id).bind(per_page).bind(offset).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let data: Vec<serde_json::Value> = items.into_iter().map(|(id, sid, map, layer, t1, t2, winner, evt, ts)| {
        serde_json::json!({ "id": id, "server_id": sid, "map_name": map, "layer_name": layer, "team1_faction": t1, "team2_faction": t2, "winner_team": winner, "event_type": evt, "logged_at": ts })
    }).collect();

    Ok(Json(serde_json::json!({ "data": data, "total": total, "page": page, "per_page": per_page })))
}

pub async fn explosion_events(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Query(q): Query<PageQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(500).min(1000);
    let offset = (page - 1) * per_page;

    let (total,) = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM explosion_events WHERE server_id=$1"
    ).bind(server_id).fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let items = sqlx::query_as::<_, (i32, i32, f64, f64, f64, String, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT id, server_id, pos_x, pos_y, pos_z, damage_causer, damage_instigator, logged_at FROM explosion_events WHERE server_id=$1 ORDER BY logged_at DESC LIMIT $2 OFFSET $3"
    ).bind(server_id).bind(per_page).bind(offset).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let data: Vec<serde_json::Value> = items.into_iter().map(|(id, sid, x, y, z, causer, instigator, ts)| {
        serde_json::json!({ "id": id, "server_id": sid, "pos_x": x, "pos_y": y, "pos_z": z, "damage_causer": causer, "damage_instigator": instigator, "logged_at": ts })
    }).collect();

    Ok(Json(serde_json::json!({ "data": data, "total": total, "page": page, "per_page": per_page })))
}

pub async fn player_info(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    Query(q): Query<PageQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(50).min(200);
    let offset = (page - 1) * per_page;

    let (items, total) = if let Some(ref name) = q.steam64 {
        let total = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM player_info WHERE server_id=$1 AND (player_name ILIKE $2 OR steam64=$2)")
            .bind(server_id).bind(format!("%{}%", name)).fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.0;
        let items = sqlx::query_as::<_, (i32, String, String, String, String, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            "SELECT server_id, player_name, steam64, eos_id, ip, first_seen, last_seen FROM player_info WHERE server_id=$1 AND (player_name ILIKE $2 OR steam64=$2) ORDER BY last_seen DESC LIMIT $3 OFFSET $4"
        ).bind(server_id).bind(format!("%{}%", name)).bind(per_page).bind(offset).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        (items, total)
    } else {
        let total = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM player_info WHERE server_id=$1")
            .bind(server_id).fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.0;
        let items = sqlx::query_as::<_, (i32, String, String, String, String, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            "SELECT server_id, player_name, steam64, eos_id, ip, first_seen, last_seen FROM player_info WHERE server_id=$1 ORDER BY last_seen DESC LIMIT $2 OFFSET $3"
        ).bind(server_id).bind(per_page).bind(offset).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        (items, total)
    };

    // 收集需要从 Steam 查询的 steam64（玩家名为空或显示为 '-'）
    let missing_ids: Vec<String> = items.iter()
        .filter(|(_, name, s64, _, _, _, _)| (name.is_empty() || name == "-") && !s64.is_empty())
        .map(|(_, _, s64, _, _, _, _)| s64.clone())
        .collect();

    // 如果有缺失的名称，调用 Steam API
    let steam_names = if !missing_ids.is_empty() && !state.steam_api_key.is_empty() {
        crate::services::steam_service::fetch_player_names(&state.steam_api_key, &missing_ids).await
    } else {
        std::collections::HashMap::new()
    };

    // 更新数据库中缺失的玩家名
    for (sid, name, s64, _, _, _, _) in &items {
        if (name.is_empty() || name == "-") && !s64.is_empty() {
            if let Some(steam_name) = steam_names.get(s64) {
                let _ = sqlx::query("UPDATE player_info SET player_name=$1 WHERE server_id=$2 AND steam64=$3")
                    .bind(steam_name).bind(sid).bind(s64).execute(&state.pool).await;
            }
        }
    }

    let data: Vec<serde_json::Value> = items.into_iter().map(|(sid, name, s64, eos, ip, fs, ls)| {
        let display_name = if name.is_empty() || name == "-" {
            steam_names.get(&s64).cloned().unwrap_or_else(|| name.clone())
        } else { name };
        serde_json::json!({ "server_id": sid, "player_name": display_name, "steam64": s64, "eos_id": eos, "ip": ip, "first_seen": fs, "last_seen": ls })
    }).collect();

    Ok(Json(serde_json::json!({ "data": data, "total": total, "page": page, "per_page": per_page })))
}

/// FOB/HAB damaged events
pub async fn deployable_events(
    State(state): State<AppState>, Path(server_id): Path<i32>, Query(q): Query<PageQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let page = q.page.unwrap_or(1).max(1); let per_page = q.per_page.unwrap_or(50).min(200);
    let offset = (page - 1) * per_page;
    let (total,) = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM deployable_damaged_events WHERE server_id=$1")
        .bind(server_id).fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let items = sqlx::query_as::<_, (i32, i32, String, f64, String, String, String, f64, chrono::DateTime<chrono::Utc>)>(
        "SELECT id,server_id,deployable,damage,weapon,player_suffix,damage_type,health_remaining,logged_at FROM deployable_damaged_events WHERE server_id=$1 ORDER BY logged_at DESC LIMIT $2 OFFSET $3"
    ).bind(server_id).bind(per_page).bind(offset).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let data: Vec<_> = items.into_iter().map(|r| serde_json::json!({"id":r.0,"server_id":r.1,"deployable":r.2,"damage":r.3,"weapon":r.4,"player_suffix":r.5,"damage_type":r.6,"health_remaining":r.7,"logged_at":r.8})).collect();
    Ok(Json(serde_json::json!({"data":data,"total":total,"page":page,"per_page":per_page})))
}

/// Server tick rate events
pub async fn tick_rate_events(
    State(state): State<AppState>, Path(server_id): Path<i32>, Query(q): Query<PageQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let page = q.page.unwrap_or(1).max(1); let per_page = q.per_page.unwrap_or(100).min(500);
    let offset = (page - 1) * per_page;
    let (total,) = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM tick_rate_events WHERE server_id=$1")
        .bind(server_id).fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let items = sqlx::query_as::<_, (i32, i32, f64, chrono::DateTime<chrono::Utc>)>(
        "SELECT id,server_id,tick_rate,logged_at FROM tick_rate_events WHERE server_id=$1 ORDER BY logged_at DESC LIMIT $2 OFFSET $3"
    ).bind(server_id).bind(per_page).bind(offset).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let data: Vec<_> = items.into_iter().map(|r| serde_json::json!({"id":r.0,"server_id":r.1,"tick_rate":r.2,"logged_at":r.3})).collect();
    Ok(Json(serde_json::json!({"data":data,"total":total,"page":page,"per_page":per_page})))
}

/// Vehicle enter/exit events
pub async fn vehicle_events(
    State(state): State<AppState>, Path(server_id): Path<i32>, Query(q): Query<PageQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let page = q.page.unwrap_or(1).max(1); let per_page = q.per_page.unwrap_or(50).min(200);
    let offset = (page - 1) * per_page;
    let (total,) = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM vehicle_events WHERE server_id=$1")
        .bind(server_id).fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let items = sqlx::query_as::<_, (i32, i32, String, String, String, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT id,server_id,player_name,steam64,vehicle_name,event_type,logged_at FROM vehicle_events WHERE server_id=$1 ORDER BY logged_at DESC LIMIT $2 OFFSET $3"
    ).bind(server_id).bind(per_page).bind(offset).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let data: Vec<_> = items.into_iter().map(|r| serde_json::json!({"id":r.0,"server_id":r.1,"player_name":r.2,"steam64":r.3,"vehicle_name":r.4,"event_type":r.5,"logged_at":r.6})).collect();
    Ok(Json(serde_json::json!({"data":data,"total":total,"page":page,"per_page":per_page})))
}

/// Player connection/disconnection events
pub async fn connection_events(
    State(state): State<AppState>, Path(server_id): Path<i32>, Query(q): Query<PageQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let page = q.page.unwrap_or(1).max(1); let per_page = q.per_page.unwrap_or(50).min(200);
    let offset = (page - 1) * per_page;

    // 尝试从 connection_events 表获取数据
    let conn_result = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM connection_events WHERE server_id=$1")
        .bind(server_id).fetch_one(&state.pool).await;

    if let Ok((total,)) = conn_result {
        if total > 0 {
            let items = sqlx::query_as::<_, (i32, String, String, String, String, chrono::DateTime<chrono::Utc>)>(
                "SELECT id,player_name,steam64,action,ip_address,logged_at FROM connection_events WHERE server_id=$1 ORDER BY logged_at DESC LIMIT $2 OFFSET $3"
            ).bind(server_id).bind(per_page).bind(offset).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let data: Vec<_> = items.into_iter().map(|r| serde_json::json!({"id":r.0,"player_name":r.1,"steam64":r.2,"action":r.3,"ip_address":r.4,"logged_at":r.5})).collect();
            return Ok(Json(serde_json::json!({"data":data,"total":total,"page":page,"per_page":per_page})));
        }
    }

    // 回退：从 player_info 获取最近活跃玩家作为连接记录
    let (total,) = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM player_info WHERE server_id=$1")
        .bind(server_id).fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let items = sqlx::query_as::<_, (i32, String, String, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT id,player_name,steam64,ip,last_seen FROM player_info WHERE server_id=$1 ORDER BY last_seen DESC LIMIT $2 OFFSET $3"
    ).bind(server_id).bind(per_page).bind(offset).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let data: Vec<_> = items.into_iter().map(|r| serde_json::json!({"id":r.0,"player_name":r.1,"steam64":r.2,"action":"connected","ip_address":r.3,"logged_at":r.4})).collect();
    Ok(Json(serde_json::json!({"data":data,"total":total,"page":page,"per_page":per_page})))
}

/// Admin broadcast messages
pub async fn admin_broadcasts(
    State(state): State<AppState>, Path(server_id): Path<i32>, Query(q): Query<PageQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let page = q.page.unwrap_or(1).max(1); let per_page = q.per_page.unwrap_or(50).min(200);
    let offset = (page - 1) * per_page;
    let (total,) = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM admin_actions WHERE server_id=$1 AND action_type='broadcast'")
        .bind(server_id).fetch_one(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let items = sqlx::query_as::<_, (String, String, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT admin_name,target,message,logged_at FROM admin_actions WHERE server_id=$1 AND action_type='broadcast' ORDER BY logged_at DESC LIMIT $2 OFFSET $3"
    ).bind(server_id).bind(per_page).bind(offset).fetch_all(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let data: Vec<_> = items.into_iter().map(|r| serde_json::json!({"admin_name":r.0,"target":r.1,"message":r.2,"logged_at":r.3})).collect();
    Ok(Json(serde_json::json!({"data":data,"total":total,"page":page,"per_page":per_page})))
}
