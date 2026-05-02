use axum::{extract::FromRequestParts, http::{request::Parts, StatusCode}, Extension};
use jsonwebtoken::{decode, DecodingKey, Validation};
use crate::api::auth::Claims;

pub struct AuthenticatedUser {
    pub username: String,
    pub role: String,
    pub permissions: serde_json::Value,
}

impl FromRequestParts<()> for AuthenticatedUser {
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _: &()) -> Result<Self, Self::Rejection> {
        let token = parts.headers.get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(|s| s.to_string())
            .unwrap_or_default();

        if token.is_empty() {
            return Err(StatusCode::UNAUTHORIZED);
        }

        let Extension(secret): Extension<String> = Extension::from_request_parts(parts, &()).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let data = decode::<Claims>(&token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::default())
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(AuthenticatedUser {
            username: data.claims.username,
            role: data.claims.role,
            permissions: data.claims.permissions,
        })
    }
}

pub fn check_perm(user: &AuthenticatedUser, key: &str) -> bool {
    if user.role == "超级管理员" { return true; }
    user.permissions.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
}
