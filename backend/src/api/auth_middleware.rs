use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    Extension,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use crate::api::auth::Claims;

#[derive(Clone)]
pub struct AuthenticatedUser {
    pub username: String,
    pub role: String,
    pub permissions: serde_json::Value,
}

fn extract_token_from_query(uri: &axum::http::Uri) -> Option<String> {
    uri.query().and_then(|q| {
        q.split('&')
            .find(|p| p.starts_with("token="))
            .map(|p| p[6..].to_string())
            .filter(|t| !t.is_empty())
    })
}

/// 认证中间件：验证 JWT 并将用户信息注入 request extensions
/// 支持两种 token 传递方式：Authorization: Bearer 头（常规 API）和 ?token= 查询参数（WebSocket 降级）
pub async fn require_auth(
    Extension(secret): Extension<String>,
    mut request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. 优先从 Authorization 头获取
    let token = request.headers().get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string())
        // 2. 降级到 URL 查询参数（用于浏览器 WebSocket 连接）
        .or_else(|| extract_token_from_query(request.uri()))
        .unwrap_or_default();

    if token.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut validation = Validation::default();
    validation.validate_exp = true;

    let data = decode::<Claims>(&token, &DecodingKey::from_secret(secret.as_bytes()), &validation)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    request.extensions_mut().insert(AuthenticatedUser {
        username: data.claims.username,
        role: data.claims.role,
        permissions: data.claims.permissions,
    });

    Ok(next.run(request).await)
}

/// 提取器：从 extensions 中读取中间件注入的用户信息（仅用于需要用户信息的 handler）
impl FromRequestParts<()> for AuthenticatedUser {
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _: &()) -> Result<Self, Self::Rejection> {
        parts.extensions.remove::<AuthenticatedUser>()
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}

pub fn check_perm(user: &AuthenticatedUser, key: &str) -> bool {
    if user.role == "超级管理员" { return true; }
    user.permissions.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
}
