use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering as AtomicOrdering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore, mpsc, oneshot};

use super::squad::SquadRcon;

// ═══ Priority Constants ═══
pub const PRIORITY_LOW: u8 = 1;
pub const PRIORITY_NORMAL: u8 = 5;
pub const PRIORITY_HIGH: u8 = 10;
pub const PRIORITY_CRITICAL: u8 = 15;

// ═══ Health Constants ═══
const MAX_CONSECUTIVE_FAILURES: u32 = 3;
const MAX_CONSECUTIVE_TIMEOUTS: u32 = 5;
const RECONNECT_DELAY_SECS: u64 = 5;
const COMMAND_QUEUE_SIZE: usize = 1000;
const DEFAULT_COMMAND_TIMEOUT_SECS: u64 = 30;

// ═══ Command Options ═══

#[derive(Clone)]
pub struct CommandOptions {
    pub priority: u8,
    pub timeout: Duration,
    pub retries: u32,
}

impl Default for CommandOptions {
    fn default() -> Self {
        Self {
            priority: PRIORITY_NORMAL,
            timeout: Duration::from_secs(DEFAULT_COMMAND_TIMEOUT_SECS),
            retries: 1,
        }
    }
}

// ═══ Queued Command (with priority ordering) ═══

struct QueuedCommand {
    command: String,
    priority: u8,
    timestamp: Instant,
    response_tx: oneshot::Sender<Result<String, String>>,
}

impl Ord for QueuedCommand {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.timestamp.cmp(&self.timestamp))
    }
}

impl PartialOrd for QueuedCommand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for QueuedCommand {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.timestamp == other.timestamp
    }
}

impl Eq for QueuedCommand {}

// ═══ Per-Server Connection State ═══

struct ServerConnection {
    ip: String,
    port: u16,
    password: Arc<Mutex<String>>,
    rcon: Mutex<Option<SquadRcon>>,
    cmd_tx: mpsc::UnboundedSender<QueuedCommand>,
    healthy: AtomicBool,
    consecutive_failures: AtomicU32,
    consecutive_timeouts: AtomicU32,
}

impl ServerConnection {
    fn spawn(ip: String, port: u16, password: String) -> Arc<Self> {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let conn = Arc::new(Self {
            ip: ip.clone(),
            port,
            password: Arc::new(Mutex::new(password.clone())),
            rcon: Mutex::new(None),
            cmd_tx,
            healthy: AtomicBool::new(true),
            consecutive_failures: AtomicU32::new(0),
            consecutive_timeouts: AtomicU32::new(0),
        });

        // Spawn command processor
        let proc_conn = conn.clone();
        tokio::spawn(async move {
            proc_conn.process_commands(cmd_rx).await;
        });

        // Spawn health monitor
        let mon_conn = conn.clone();
        tokio::spawn(async move {
            mon_conn.monitor_health().await;
        });

        conn
    }

    fn submit(&self, cmd: QueuedCommand) -> Result<(), QueuedCommand> {
        self.cmd_tx.send(cmd).map_err(|e| e.0)
    }

    async fn update_password(&self, new_password: &str) {
        let mut pw = self.password.lock().await;
        if *pw != new_password {
            *pw = new_password.to_string();
            // Force reconnect on next command by closing current connection
            let mut rcon = self.rcon.lock().await;
            *rcon = None;
            self.healthy.store(false, AtomicOrdering::SeqCst);
        }
    }

    // ═══ Command Processor ═══

    async fn process_commands(self: Arc<Self>, mut rx: mpsc::UnboundedReceiver<QueuedCommand>) {
        let semaphore = Semaphore::new(1);
        let mut heap: BinaryHeap<QueuedCommand> = BinaryHeap::new();

        loop {
            // Drain any queued commands before blocking
            while let Ok(cmd) = rx.try_recv() {
                heap.push(cmd);
            }

            if heap.is_empty() {
                match rx.recv().await {
                    Some(cmd) => heap.push(cmd),
                    None => break,
                }
                continue;
            }

            // Acquire semaphore — ensures only 1 command executes at a time
            let permit = semaphore.acquire().await;

            // Drain again (more may have arrived during semaphore wait)
            while let Ok(cmd) = rx.try_recv() {
                heap.push(cmd);
            }

            if let Some(cmd) = heap.pop() {
                let response_tx = cmd.response_tx;
                let result = self.execute_internal(&cmd.command).await;
                self.record_result(&result);
                let _ = response_tx.send(result);
            }

            drop(permit);
        }
    }

    async fn execute_internal(&self, command: &str) -> Result<String, String> {
        // Ensure we have a healthy connection
        self.ensure_connected().await?;

        let mut rcon_guard = self.rcon.lock().await;
        if let Some(ref mut rcon) = *rcon_guard {
            match rcon.execute(command).await {
                Ok(resp) => Ok(resp),
                Err(e) => {
                    // Connection died — remove and let next command reconnect
                    *rcon_guard = None;
                    Err(e)
                }
            }
        } else {
            Err("RCON 未连接".to_string())
        }
    }

    async fn ensure_connected(&self) -> Result<(), String> {
        {
            let rcon = self.rcon.lock().await;
            if rcon.is_some() {
                return Ok(());
            }
        }

        // Need to connect
        let password = {
            let pw = self.password.lock().await;
            pw.clone()
        };

        match SquadRcon::connect(&self.ip, self.port, &password).await {
            Ok(rcon) => {
                let mut guard = self.rcon.lock().await;
                *guard = Some(rcon);
                self.healthy.store(true, AtomicOrdering::SeqCst);
                self.consecutive_failures.store(0, AtomicOrdering::SeqCst);
                self.consecutive_timeouts.store(0, AtomicOrdering::SeqCst);
                Ok(())
            }
            Err(e) => {
                self.healthy.store(false, AtomicOrdering::SeqCst);
                Err(format!("RCON 连接失败: {}", e))
            }
        }
    }

    // ═══ Health Tracking ═══

    fn record_result(&self, result: &Result<String, String>) {
        match result {
            Ok(_) => {
                self.healthy.store(true, AtomicOrdering::SeqCst);
                self.consecutive_failures.store(0, AtomicOrdering::SeqCst);
                self.consecutive_timeouts.store(0, AtomicOrdering::SeqCst);
            }
            Err(e) => {
                let is_timeout = e.contains("超时") || e.contains("timeout") || e.contains("Timeout");
                if is_timeout {
                    let timeouts = self
                        .consecutive_timeouts
                        .fetch_add(1, AtomicOrdering::SeqCst)
                        + 1;
                    if timeouts >= MAX_CONSECUTIVE_TIMEOUTS {
                        self.healthy.store(false, AtomicOrdering::SeqCst);
                    }
                } else {
                    // Non-timeout error — reset timeout counter, increment failure counter
                    self.consecutive_timeouts.store(0, AtomicOrdering::SeqCst);
                    let failures = self
                        .consecutive_failures
                        .fetch_add(1, AtomicOrdering::SeqCst)
                        + 1;
                    if failures >= MAX_CONSECUTIVE_FAILURES {
                        self.healthy.store(false, AtomicOrdering::SeqCst);
                    }
                }
            }
        }
    }

    // ═══ Health Monitor ═══

    async fn monitor_health(self: Arc<Self>) {
        loop {
            tokio::time::sleep(Duration::from_secs(RECONNECT_DELAY_SECS)).await;

            if !self.healthy.load(AtomicOrdering::SeqCst) {
                tracing::info!(
                    ip = %self.ip,
                    port = %self.port,
                    "RCON 连接不健康，尝试重连"
                );

                // Close old connection
                {
                    let mut rcon = self.rcon.lock().await;
                    *rcon = None;
                }

                // Attempt reconnect
                let password = {
                    let pw = self.password.lock().await;
                    pw.clone()
                };

                match SquadRcon::connect(&self.ip, self.port, &password).await {
                    Ok(new_rcon) => {
                        let mut guard = self.rcon.lock().await;
                        *guard = Some(new_rcon);
                        self.healthy.store(true, AtomicOrdering::SeqCst);
                        self.consecutive_failures.store(0, AtomicOrdering::SeqCst);
                        self.consecutive_timeouts.store(0, AtomicOrdering::SeqCst);
                        tracing::info!(
                            ip = %self.ip,
                            port = %self.port,
                            "RCON 重连成功"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            ip = %self.ip,
                            port = %self.port,
                            error = %e,
                            "RCON 重连失败，将在 {} 秒后重试",
                            RECONNECT_DELAY_SECS
                        );
                    }
                }
            }
        }
    }

    fn is_healthy(&self) -> bool {
        self.healthy.load(AtomicOrdering::SeqCst)
    }
}

// ═══ Public Pool API ═══

#[derive(Clone)]
pub struct RconPool {
    connections: Arc<Mutex<std::collections::HashMap<String, Arc<ServerConnection>>>>,
}

impl RconPool {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Execute with default options (normal priority, 30s timeout, 1 retry)
    pub async fn execute(
        &self,
        ip: &str,
        port: u16,
        password: &str,
        command: &str,
    ) -> Result<String, String> {
        self.execute_with_options(ip, port, password, command, CommandOptions::default())
            .await
    }

    /// Execute with high priority (for automated services like damage_notify)
    pub async fn execute_high_priority(
        &self,
        ip: &str,
        port: u16,
        password: &str,
        command: &str,
    ) -> Result<String, String> {
        self.execute_with_options(
            ip,
            port,
            password,
            command,
            CommandOptions {
                priority: PRIORITY_HIGH,
                ..Default::default()
            },
        )
        .await
    }

    /// Execute with critical priority (for ban enforcement etc.)
    pub async fn execute_critical(
        &self,
        ip: &str,
        port: u16,
        password: &str,
        command: &str,
    ) -> Result<String, String> {
        self.execute_with_options(
            ip,
            port,
            password,
            command,
            CommandOptions {
                priority: PRIORITY_CRITICAL,
                ..Default::default()
            },
        )
        .await
    }

    /// Execute with full options including retry logic
    pub async fn execute_with_options(
        &self,
        ip: &str,
        port: u16,
        password: &str,
        command: &str,
        options: CommandOptions,
    ) -> Result<String, String> {
        let key = format!("{}:{}", ip, port);
        let conn = self.get_or_create_connection(ip, port, password).await?;

        let mut last_err = String::new();
        let max_attempts = options.retries + 1;

        for attempt in 0..max_attempts {
            if attempt > 0 {
                tracing::debug!(
                    ip = %ip, port = %port, attempt,
                    "RCON 命令重试"
                );
            }

            let (response_tx, response_rx) = oneshot::channel();
            let cmd = QueuedCommand {
                command: command.to_string(),
                priority: options.priority,
                timestamp: Instant::now(),
                response_tx,
            };

            match conn.submit(cmd) {
                Ok(()) => {}
                Err(_) => {
                    // Connection's command channel closed — remove and recreate
                    let mut map = self.connections.lock().await;
                    map.remove(&key);
                    drop(map);
                    return Err("RCON 连接队列已关闭".to_string());
                }
            }

            // Await response with timeout
            match tokio::time::timeout(options.timeout, response_rx).await {
                Ok(Ok(Ok(response))) => return Ok(response),
                Ok(Ok(Err(e))) => {
                    last_err = e;
                    // Connection errors trigger immediate retry
                    if last_err.contains("连接失败") || last_err.contains("未连接") {
                        continue;
                    }
                    // Non-connection errors don't benefit from retry
                    break;
                }
                Ok(Err(_)) => {
                    last_err = "内部错误: 响应通道已关闭".to_string();
                    break;
                }
                Err(_) => {
                    last_err = format!("RCON 命令超时 ({}s)", options.timeout.as_secs());
                    // Timeout triggers retry
                    continue;
                }
            }
        }

        Err(format!(
            "RCON 命令失败 ({} 次尝试后): {}",
            max_attempts, last_err
        ))
    }

    /// Check if a server's connection is healthy
    pub async fn is_healthy(&self, ip: &str, port: u16) -> bool {
        let key = format!("{}:{}", ip, port);
        let map = self.connections.lock().await;
        map.get(&key)
            .map(|c| c.is_healthy())
            .unwrap_or(false)
    }

    /// Force reconnect for a server (e.g., after password change)
    pub async fn force_reconnect(&self, ip: &str, port: u16) {
        let key = format!("{}:{}", ip, port);
        let map = self.connections.lock().await;
        if let Some(conn) = map.get(&key) {
            let mut rcon = conn.rcon.lock().await;
            *rcon = None;
            conn.healthy.store(false, AtomicOrdering::SeqCst);
            tracing::info!(ip = %ip, port = %port, "已强制断开 RCON 连接");
        }
    }

    /// Get the number of active connections
    pub async fn connection_count(&self) -> usize {
        let map = self.connections.lock().await;
        map.len()
    }

    // ═══ Internal ═══

    async fn get_or_create_connection(
        &self,
        ip: &str,
        port: u16,
        password: &str,
    ) -> Result<Arc<ServerConnection>, String> {
        let key = format!("{}:{}", ip, port);

        // Fast path: connection exists
        {
            let map = self.connections.lock().await;
            if let Some(conn) = map.get(&key) {
                // Check if password changed
                let needs_update = {
                    let stored_pw = conn.password.lock().await;
                    *stored_pw != password
                };
                if needs_update {
                    // Clone Arc before dropping map lock
                    let conn_clone = conn.clone();
                    drop(map);
                    conn_clone.update_password(password).await;
                } else {
                    return Ok(conn.clone());
                }
            }
        }

        // Slow path: create new connection
        let conn = ServerConnection::spawn(ip.to_string(), port, password.to_string());
        let mut map = self.connections.lock().await;
        map.insert(key, conn.clone());
        Ok(conn)
    }
}
