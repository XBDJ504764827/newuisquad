use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    Extension,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use crate::api::auth::Claims;
use crate::api::AppState;
use std::time::Instant;

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

/// 缓存条目：版本号 + 写入时间
pub struct CacheEntry {
    pub version: i32,
    pub created_at: Instant,
}

const CACHE_TTL_SECS: u64 = 60;

/// 认证中间件：验证 JWT 并校验权限版本号
/// 支持两种 token 传递方式：Authorization: Bearer 头（常规 API）和 ?token= 查询参数（WebSocket 降级）
pub async fn require_auth(
    Extension(secret): Extension<String>,
    Extension(app_state): Extension<AppState>,
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

    let claims = &data.claims;

    // 3. 校验权限版本号
    let db_version = get_permission_version(&app_state, &claims.username).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    if claims.permission_version != db_version {
        // 版本不匹配 → 权限已变更，拒绝旧 token
        return Err(StatusCode::UNAUTHORIZED);
    }

    request.extensions_mut().insert(AuthenticatedUser {
        username: claims.username.clone(),
        role: claims.role.clone(),
        permissions: claims.permissions.clone(),
    });

    Ok(next.run(request).await)
}

/// 获取用户的权限版本号（内存缓存 + DB 回退）
async fn get_permission_version(state: &AppState, username: &str) -> Result<i32, String> {
    let now = Instant::now();

    // 先查缓存
    {
        let cache = state.permission_version_cache.read().await;
        if let Some(entry) = cache.get(username) {
            if now.duration_since(entry.created_at).as_secs() < CACHE_TTL_SECS {
                return Ok(entry.version);
            }
        }
    }

    // 缓存未命中或已过期 → 查 DB
    let row = sqlx::query_as::<_, (i32,)>(
        "SELECT permission_version FROM admin_users WHERE username=$1"
    ).bind(username).fetch_optional(&state.pool).await
        .map_err(|e| format!("查询权限版本失败: {}", e))?;

    let version = match row {
        Some((v,)) => v,
        None => return Err("用户不存在".to_string()),
    };

    // 写入缓存
    {
        let mut cache = state.permission_version_cache.write().await;
        cache.insert(username.to_string(), CacheEntry {
            version,
            created_at: now,
        });
        // 缓存清理：超过 1000 条时清除过期条目
        if cache.len() > 1000 {
            cache.retain(|_, entry| now.duration_since(entry.created_at).as_secs() < CACHE_TTL_SECS);
        }
    }

    Ok(version)
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
