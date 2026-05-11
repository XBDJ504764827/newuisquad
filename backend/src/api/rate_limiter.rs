use axum::{
    extract::ConnectInfo,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Extension,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

use crate::redis::RedisClient;

const WINDOW_SECS: u64 = 60;
const MAX_REQUESTS: u32 = 120;
const LOGIN_MAX_REQUESTS: u32 = 10;

/// 从请求中提取真实客户端 IP
/// 优先级：X-Forwarded-For > X-Real-IP > 直连 SocketAddr
fn extract_real_ip<B>(request: &axum::http::Request<B>, addr: SocketAddr) -> String {
    let headers = request.headers();

    if let Some(ip) = headers.get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim().to_string())
    {
        if !ip.is_empty() {
            return ip;
        }
    }

    if let Some(ip) = headers.get("X-Real-IP")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
    {
        if !ip.is_empty() {
            return ip;
        }
    }

    addr.ip().to_string()
}

/// 内存回退——基于真实 IP 的固定窗口速率限制
pub struct RateLimiterInner {
    entries: Mutex<HashMap<String, Vec<Instant>>>,
}

impl RateLimiterInner {
    fn new() -> Self {
        Self { entries: Mutex::new(HashMap::new()) }
    }

    async fn check(&self, ip: &str, max_req: u32) -> bool {
        let now = Instant::now();
        let window = std::time::Duration::from_secs(WINDOW_SECS);
        let mut map = self.entries.lock().await;
        let timestamps = map.entry(ip.to_string()).or_default();
        timestamps.retain(|t| now.duration_since(*t) < window);
        if timestamps.len() >= max_req as usize {
            false
        } else {
            timestamps.push(now);
            if map.len() > 10000 {
                map.retain(|_, ts| {
                    ts.retain(|t| now.duration_since(*t) < window);
                    !ts.is_empty()
                });
            }
            true
        }
    }
}

#[derive(Clone)]
pub struct RateLimiterState {
    pub inner: Arc<RateLimiterInner>,
    pub redis: RedisClient,
}

impl RateLimiterState {
    pub fn new(redis: RedisClient) -> Self {
        Self { inner: Arc::new(RateLimiterInner::new()), redis }
    }

    /// 统一速率检查入口：优先用 Redis 滑动窗口，回退到内存
    async fn check_rate_limit(&self, ip: &str, max_req: u32) -> bool {
        if let Ok(false) = self.redis.check_rate_limit(ip, max_req, WINDOW_SECS).await {
            return false;
        }
        // Redis Disabled 或成功 → 内存回退 / 补充
        self.inner.check(ip, max_req).await
    }
}

pub async fn rate_limit(
    Extension(limiter): Extension<RateLimiterState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Response {
    let real_ip = extract_real_ip(&request, addr);
    if !limiter.check_rate_limit(&real_ip, MAX_REQUESTS).await {
        return (StatusCode::TOO_MANY_REQUESTS, "请求过于频繁，请稍后再试").into_response();
    }
    next.run(request).await
}

pub async fn login_rate_limit(
    Extension(limiter): Extension<RateLimiterState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Response {
    let real_ip = extract_real_ip(&request, addr);
    if !limiter.check_rate_limit(&real_ip, LOGIN_MAX_REQUESTS).await {
        return (StatusCode::TOO_MANY_REQUESTS, "登录尝试过于频繁，请稍后再试").into_response();
    }
    next.run(request).await
}
