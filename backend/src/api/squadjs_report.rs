use axum::{Json, extract::{State, Path}, http::StatusCode};
use serde::Deserialize;
use crate::api::AppState;

#[derive(Deserialize)]
pub struct SquadJsReport {
    pub squads: Vec<SquadJsSquad>,
    pub map_name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SquadJsSquad {
    pub id: i32,
    pub name: String,
    #[serde(default)]
    pub size: i32,
    #[serde(alias = "teamID")]
    pub team_id: Option<i32>,
    pub leader: Option<SquadJsLeader>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SquadJsLeader {
    pub name: String,
    #[serde(alias = "steamID")]
    pub steam_id: Option<String>,
}

pub async fn handler(
    State(state): State<AppState>,
    Path(server_id): Path<i32>,
    bearer: axum::http::HeaderMap,
    Json(report): Json<SquadJsReport>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // 验证 Bearer token
    let token = bearer
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");

    let valid = sqlx::query_scalar::<_, i32>("SELECT id FROM servers WHERE id=$1 AND token=$2")
        .bind(server_id)
        .bind(token)
        .fetch_optional(&state.pool)
        .await
        .map(|r| r.is_some())
        .unwrap_or(false);

    if !valid {
        return Ok(Json(serde_json::json!({ "error": "认证失败" })));
    }

    let tid = server_id.to_string();

    // 更新缓存：添加 squadjs_squads（含队长信息）和 squadjs_map_name
    let mut cache = state.server_states.write().await;
        let entry = cache.entry(tid).or_insert(serde_json::json!({}));

        if let Some(obj) = entry.as_object_mut() {
            // 更新小队数据（含队长信息）
            let squads_json: Vec<serde_json::Value> = report.squads.iter().map(|s| {
                let team_id = s.team_id.unwrap_or(0);
                serde_json::json!({
                    "id": s.id,
                    "name": s.name,
                    "size": s.size,
                    "team_id": team_id,
                    "leader": s.leader.as_ref().map(|l| serde_json::json!({
                        "name": l.name,
                        "steam_id": l.steam_id.clone().unwrap_or_default(),
                    })),
                })
            }).collect();
            obj.insert("squadjs_squads".into(), serde_json::Value::Array(squads_json));

            // 更新地图名称
            if !report.map_name.is_empty() {
                obj.insert("map_name".into(), serde_json::json!(report.map_name));
            }
        }

    tracing::info!(server_id, squads = report.squads.len(), map = %report.map_name, "SquadJS 数据上报");

    Ok(Json(serde_json::json!({ "success": true })))
}
