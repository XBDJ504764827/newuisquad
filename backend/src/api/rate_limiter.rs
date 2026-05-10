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

const WINDOW_SECS: u64 = 60;
const MAX_REQUESTS: u32 = 120;
const LOGIN_MAX_REQUESTS: u32 = 10;

pub struct RateLimiterInner {
    entries: Mutex<HashMap<SocketAddr, Vec<Instant>>>,
}

impl RateLimiterInner {
    fn new() -> Self {
        Self { entries: Mutex::new(HashMap::new()) }
    }

    async fn check(&self, addr: SocketAddr, max_req: u32) -> bool {
        let now = Instant::now();
        let window = std::time::Duration::from_secs(WINDOW_SECS);
        let mut map = self.entries.lock().await;
        let timestamps = map.entry(addr).or_default();
        timestamps.retain(|t| now.duration_since(*t) < window);
        if timestamps.len() >= max_req as usize {
            false
        } else {
            timestamps.push(now);
            // 内存清理：条目数超过 10000 时清除过期 IP
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
}

impl RateLimiterState {
    pub fn new() -> Self {
        Self { inner: Arc::new(RateLimiterInner::new()) }
    }
}

pub async fn rate_limit(
    Extension(limiter): Extension<RateLimiterState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Response {
    if !limiter.inner.check(addr, MAX_REQUESTS).await {
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
    if !limiter.inner.check(addr, LOGIN_MAX_REQUESTS).await {
        return (StatusCode::TOO_MANY_REQUESTS, "登录尝试过于频繁，请稍后再试").into_response();
    }
    next.run(request).await
}
