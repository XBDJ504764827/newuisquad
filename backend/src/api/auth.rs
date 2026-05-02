use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use chrono::Utc;
use crate::api::AppState;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub username: String,
    pub role: String,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub role: String,
    pub exp: usize,
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = sqlx::query_as::<_, (i32, String, String, String)>(
        "SELECT id, username, password_hash, role FROM admin_users WHERE username=$1"
    ).bind(&req.username).fetch_optional(&state.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (_, username, password_hash, role) = match user {
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
        exp: (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(state.jwt_secret.as_bytes()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    crate::services::system_log::action_log(&state.pool, "auth", &format!("用户 {} 登录", username), "").await;

    Ok(Json(serde_json::json!({ "token": token, "username": username, "role": role })))
}

pub async fn verify_token(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let token = req["token"].as_str().unwrap_or("");
    match decode::<Claims>(token, &DecodingKey::from_secret(state.jwt_secret.as_bytes()), &Validation::default()) {
        Ok(data) => Ok(Json(serde_json::json!({ "valid": true, "username": data.claims.username, "role": data.claims.role }))),
        Err(_) => Ok(Json(serde_json::json!({ "valid": false }))),
    }
}
