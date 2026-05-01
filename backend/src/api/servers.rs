use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use sqlx::Row;
use crate::api::AppState;
use crate::rcon_client::squad::SquadRcon;

#[derive(Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
    pub ip: String,
    pub rcon_port: i32,
    pub rcon_password: String,
    #[serde(default = "default_admin")]
    pub admin_user: String,
}

fn default_admin() -> String { "Admin".to_string() }

pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateServerRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // 1. RCON 验证
    match SquadRcon::connect(&req.ip, req.rcon_port as u16, &req.rcon_password).await {
        Ok(mut rcon) => {
            if let Err(e) = rcon.execute("ping").await {
                return Ok(Json(serde_json::json!({ "error": format!("RCON 命令测试失败: {}", e) })));
            }
        }
        Err(e) => {
            return Ok(Json(serde_json::json!({ "error": format!("RCON 连接失败: {}", e) })));
        }
    }

    // 2. 生成 server_id 和 token
    let rid: u32 = rand::random();
    let server_id = format!("SRV-{}", hex::encode(rid.to_le_bytes()).to_uppercase());
    let rtoken: u64 = rand::random();
    let words = ["blind", "sdk", "upgrade", "crash", "panel", "quick", "zone", "flux"];
    let word = words[(rand::random::<u32>() as usize) % words.len()];
    let token = format!("sk_{}_{}", word, hex::encode(rtoken.to_le_bytes()));

    // 3. 写入数据库
    let result = sqlx::query(
        "INSERT INTO servers (server_id, name, ip, rcon_port, rcon_password, token) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, server_id, name, ip, rcon_port, created_at"
    )
    .bind(&server_id).bind(&req.name).bind(&req.ip).bind(req.rcon_port).bind(&req.rcon_password).bind(&token)
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok(row) => {
            Ok(Json(serde_json::json!({
                "id": row.get::<i32, _>(0),
                "server_id": row.get::<String, _>(1),
                "name": row.get::<String, _>(2),
                "ip": row.get::<String, _>(3),
                "rcon_port": row.get::<i32, _>(4),
                "token": token,
                "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>(5),
            })))
        }
        Err(e) => Ok(Json(serde_json::json!({ "error": format!("保存失败: {}", e) }))),
    }
}
