use redis::aio::ConnectionManager;
use uuid::Uuid;

/// Redis 客户端封装 — Connected 使用 Redis 连接，Disabled 优雅降级
#[derive(Clone)]
pub enum RedisClient {
    Connected(ConnectionManager),
    Disabled,
}

impl RedisClient {
    /// 创建 Redis 客户端
    /// - 传入 Some(url) 且连接成功 → Connected
    /// - 传入 None / 空串 / 连接失败 → Disabled（服务仍正常运行）
    pub async fn new(redis_url: Option<&str>) -> Self {
        match redis_url {
            Some(url) if !url.is_empty() => {
                match redis::Client::open(url) {
                    Ok(client) => match ConnectionManager::new(client).await {
                        Ok(manager) => {
                            tracing::info!("Redis 已连接: {}", url);
                            RedisClient::Connected(manager)
                        }
                        Err(e) => {
                            tracing::warn!("Redis 连接失败 ({}), 回退到内存模式", e);
                            RedisClient::Disabled
                        }
                    },
                    Err(e) => {
                        tracing::warn!("Redis URL 解析失败 ({}), 回退到内存模式", e);
                        RedisClient::Disabled
                    }
                }
            }
            _ => {
                tracing::info!("Redis 未配置，使用内存模式");
                RedisClient::Disabled
            }
        }
    }

    pub fn is_connected(&self) -> bool {
        matches!(self, RedisClient::Connected(_))
    }

    // ════════════════════════════════════════════
    //  速率限制 — ZSET 滑动窗口算法
    // ════════════════════════════════════════════

    /// 检查速率限制。返回 Ok(true) 表示允许，Ok(false) 表示超出限制。
    pub async fn check_rate_limit(
        &self,
        ip_key: &str,
        max_requests: u32,
        window_secs: u64,
    ) -> Result<bool, ()> {
        match self {
            RedisClient::Connected(conn) => {
                let mut con = conn.clone();
                let redis_key = format!("ratelimit:{}", ip_key);
                let now = chrono::Utc::now().timestamp_millis() as f64;
                let window_start = now - (window_secs as f64 * 1000.0);

                // 1. 移除窗口外的旧记录
                let _: () = redis::cmd("ZREMRANGEBYSCORE")
                    .arg(&redis_key)
                    .arg("-inf")
                    .arg(window_start)
                    .query_async(&mut con)
                    .await
                    .map_err(|e| tracing::error!(error = %e, "Redis ZREMRANGEBYSCORE 失败"))?;

                // 2. 计数当前窗口内请求数
                let count: u32 = redis::cmd("ZCARD")
                    .arg(&redis_key)
                    .query_async(&mut con)
                    .await
                    .map_err(|e| tracing::error!(error = %e, "Redis ZCARD 失败"))?;

                if count >= max_requests {
                    return Ok(false);
                }

                // 3. 添加当前请求（时间戳+UUID 保证唯一）
                let member = format!("{}-{}", now as u64, Uuid::new_v4());
                let _: () = redis::cmd("ZADD")
                    .arg(&redis_key)
                    .arg(now)
                    .arg(&member)
                    .query_async(&mut con)
                    .await
                    .map_err(|e| tracing::error!(error = %e, "Redis ZADD 失败"))?;

                // 4. 设置过期时间（窗口2倍，防止残留）
                let _: () = redis::cmd("EXPIRE")
                    .arg(&redis_key)
                    .arg((window_secs * 2) as i64)
                    .query_async(&mut con)
                    .await
                    .map_err(|e| tracing::error!(error = %e, "Redis EXPIRE 失败"))?;

                Ok(true)
            }
            RedisClient::Disabled => Ok(true), // 回退：上层由内存限制器接管
        }
    }

    // ════════════════════════════════════════════
    //  JWT 黑名单
    // ════════════════════════════════════════════

    /// 将 JTI 加入黑名单，TTL 对齐 token 剩余有效期
    pub async fn add_to_blacklist(&self, jti: &str, ttl_secs: u64) -> Result<(), ()> {
        match self {
            RedisClient::Connected(conn) => {
                let mut con = conn.clone();
                let key = format!("jwt:blacklist:{}", jti);
                redis::cmd("SETEX")
                    .arg(&key)
                    .arg(ttl_secs)
                    .arg("1")
                    .query_async::<()>(&mut con)
                    .await
                    .map_err(|e| tracing::error!(error = %e, "Redis SETEX 黑名单失败"))
            }
            RedisClient::Disabled => Ok(()),
        }
    }

    /// 检查 JTI 是否已被拉黑
    pub async fn is_blacklisted(&self, jti: &str) -> Result<bool, ()> {
        match self {
            RedisClient::Connected(conn) => {
                let mut con = conn.clone();
                let key = format!("jwt:blacklist:{}", jti);
                redis::cmd("EXISTS")
                    .arg(&key)
                    .query_async::<i32>(&mut con)
                    .await
                    .map(|n| n > 0)
                    .map_err(|e| tracing::error!(error = %e, "Redis EXISTS 黑名单检查失败"))
            }
            RedisClient::Disabled => Ok(false),
        }
    }

    // ════════════════════════════════════════════
    //  通用缓存
    // ════════════════════════════════════════════

    pub async fn cache_get(&self, namespace: &str, key: &str) -> Result<Option<String>, ()> {
        match self {
            RedisClient::Connected(conn) => {
                let mut con = conn.clone();
                let redis_key = format!("cache:{}:{}", namespace, key);
                redis::cmd("GET")
                    .arg(&redis_key)
                    .query_async::<Option<String>>(&mut con)
                    .await
                    .map_err(|e| tracing::error!(error = %e, "Redis GET 失败"))
            }
            RedisClient::Disabled => Ok(None),
        }
    }

    pub async fn cache_set(&self, namespace: &str, key: &str, value: &str, ttl_secs: u64) -> Result<(), ()> {
        match self {
            RedisClient::Connected(conn) => {
                let mut con = conn.clone();
                let redis_key = format!("cache:{}:{}", namespace, key);
                redis::cmd("SETEX")
                    .arg(&redis_key)
                    .arg(ttl_secs)
                    .arg(value)
                    .query_async::<()>(&mut con)
                    .await
                    .map_err(|e| tracing::error!(error = %e, "Redis SETEX 缓存失败"))
            }
            RedisClient::Disabled => Ok(()),
        }
    }

    pub async fn cache_del(&self, namespace: &str, key: &str) -> Result<(), ()> {
        match self {
            RedisClient::Connected(conn) => {
                let mut con = conn.clone();
                let redis_key = format!("cache:{}:{}", namespace, key);
                redis::cmd("DEL")
                    .arg(&redis_key)
                    .query_async::<()>(&mut con)
                    .await
                    .map_err(|e| tracing::error!(error = %e, "Redis DEL 失败"))
            }
            RedisClient::Disabled => Ok(()),
        }
    }

    /// 缓存一个整数
    pub async fn cache_get_i64(&self, namespace: &str, key: &str) -> Result<Option<i64>, ()> {
        match self {
            RedisClient::Connected(conn) => {
                let mut con = conn.clone();
                let redis_key = format!("cache:{}:{}", namespace, key);
                redis::cmd("GET")
                    .arg(&redis_key)
                    .query_async::<Option<i64>>(&mut con)
                    .await
                    .map_err(|e| tracing::error!(error = %e, "Redis GET i64 失败"))
            }
            RedisClient::Disabled => Ok(None),
        }
    }

    pub async fn cache_set_i64(&self, namespace: &str, key: &str, value: i64, ttl_secs: u64) -> Result<(), ()> {
        match self {
            RedisClient::Connected(conn) => {
                let mut con = conn.clone();
                let redis_key = format!("cache:{}:{}", namespace, key);
                redis::cmd("SETEX")
                    .arg(&redis_key)
                    .arg(ttl_secs)
                    .arg(value)
                    .query_async::<()>(&mut con)
                    .await
                    .map_err(|e| tracing::error!(error = %e, "Redis SETEX i64 失败"))
            }
            RedisClient::Disabled => Ok(()),
        }
    }

    /// 缓存序列化的 JSON 值
    pub async fn cache_get_json<T: serde::de::DeserializeOwned>(&self, namespace: &str, key: &str) -> Result<Option<T>, ()> {
        match self {
            RedisClient::Connected(conn) => {
                let raw = self.cache_get(namespace, key).await?;
                match raw {
                    Some(s) => serde_json::from_str(&s).map(Some).map_err(|_| ()),
                    None => Ok(None),
                }
            }
            RedisClient::Disabled => Ok(None),
        }
    }

    pub async fn cache_set_json<T: serde::Serialize>(&self, namespace: &str, key: &str, value: &T, ttl_secs: u64) -> Result<(), ()> {
        let json = serde_json::to_string(value).map_err(|_| ())?;
        self.cache_set(namespace, key, &json, ttl_secs).await
    }
}
