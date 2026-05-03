use axum::{Json, extract::State, http::{StatusCode, HeaderMap}};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use chrono::Utc;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use crate::api::AppState;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub role: String,
    pub permissions: JsonValue,
    pub exp: usize,
}

/// 简易 IP 速率限制器：每 IP 每分钟最多 5 次失败登录
struct RateLimiter {
    attempts: Mutex<HashMap<String, Vec<Instant>>>,
}

impl RateLimiter {
    fn new() -> Self {
        Self { attempts: Mutex::new(HashMap::new()) }
    }

    fn check(&self, ip: &str) -> bool {
        let mut map = self.attempts.lock().unwrap();
        let now = Instant::now();
        let window = now - std::time::Duration::from_secs(60);
        let entries = map.entry(ip.to_string()).or_default();
        entries.retain(|t| *t > window);
        entries.push(now);
        if entries.len() > 100 { entries.drain(..50); }
        entries.len() <= 5
    }
}

static RATE_LIMITER: std::sync::OnceLock<RateLimiter> = std::sync::OnceLock::new();

fn rate_limiter() -> &'static RateLimiter {
    RATE_LIMITER.get_or_init(|| RateLimiter::new())
}

fn client_ip(headers: &HeaderMap) -> String {
    if let Some(ip) = headers.get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
    {
        return ip.trim().to_string();
    }
    if let Some(ip) = headers.get("X-Real-IP").and_then(|v| v.to_str().ok()) {
        return ip.trim().to_string();
    }
    "unknown".to_string()
}

pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let ip = client_ip(&headers);

    if !rate_limiter().check(&ip) {
        return Ok(Json(serde_json::json!({ "error": "尝试次数过多，请 1 分钟后重试" })));
    }

    let user = sqlx::query_as::<_, (i32, String, String, String, JsonValue)>(
        "SELECT id, username, password_hash, role, permissions FROM admin_users WHERE username=$1"
    ).bind(&req.username).fetch_optional(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (_, username, password_hash, role, permissions) = match user {
        Some(u) => u,
        None => return Ok(Json(serde_json::json!({ "error": "用户名或密码错误" }))),
    };

    if !bcrypt::verify(&req.password, &password_hash).unwrap_or(false) {
        return Ok(Json(serde_json::json!({ "error": "用户名或密码错误" })));
    }

    let claims = Claims {
        sub: username.clone(),
        username: username.clone(),
        role: role.clone(),
        permissions: permissions.clone(),
        exp: (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let mut validation = Validation::default();
    validation.validate_exp = true;

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(state.jwt_secret.as_bytes()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    crate::services::system_log::action_log(&state.pool, "auth", &format!("用户 {} 登录", username), &ip).await;

    Ok(Json(serde_json::json!({ "token": token, "username": username, "role": role, "permissions": permissions })))
}

pub async fn verify_token(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let token = req["token"].as_str().unwrap_or("");
    let mut validation = Validation::default();
    validation.validate_exp = true;
    match decode::<Claims>(token, &DecodingKey::from_secret(state.jwt_secret.as_bytes()), &validation) {
        Ok(data) => Ok(Json(serde_json::json!({ "valid": true, "username": data.claims.username, "role": data.claims.role, "permissions": data.claims.permissions }))),
        Err(_) => Ok(Json(serde_json::json!({ "valid": false }))),
    }
}
