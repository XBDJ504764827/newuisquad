use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::interval;

use crate::clickhouse::{ClickHousePool, schema::*};

const BATCH_SIZE: usize = 1000;
const FLUSH_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub enum GameEvent {
    Chat(ChatMessageEvent),
    Damaged(PlayerDamagedEvent),
    Died(PlayerDiedEvent),
    Connected(PlayerConnectedEvent),
    Deployable(DeployableEvent),
    TickRate(TickRateEvent),
    Match(MatchEvent),
    Vehicle(VehicleEvent),
    VehicleDamage(VehicleDamageEvent),
    Fly(FlyEvent),
}

pub struct ClickHouseIngestService {
    pool: Arc<ClickHousePool>,
    buffer: Mutex<Vec<GameEvent>>,
    enabled: bool,
}

impl ClickHouseIngestService {
    pub fn new(pool: Arc<ClickHousePool>) -> Self {
        Self {
            pool,
            buffer: Mutex::new(Vec::with_capacity(BATCH_SIZE)),
            enabled: true,
        }
    }

    pub fn disabled() -> Self {
        Self {
            pool: Arc::new(ClickHousePool {
                client: Arc::new(clickhouse::Client::default()),
                database: String::new(),
            }),
            buffer: Mutex::new(Vec::new()),
            enabled: false,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub async fn ingest_event(&self, event: GameEvent) {
        if !self.enabled {
            return;
        }
        let mut buffer = self.buffer.lock().await;
        buffer.push(event);
        if buffer.len() >= BATCH_SIZE {
            let events = std::mem::take(&mut *buffer);
            drop(buffer);
            self.flush_batch(events).await;
        }
    }

    pub async fn ingest_events(&self, events: Vec<GameEvent>) {
        if !self.enabled || events.is_empty() {
            return;
        }
        let mut buffer = self.buffer.lock().await;
        buffer.extend(events);
        if buffer.len() >= BATCH_SIZE {
            let events = std::mem::take(&mut *buffer);
            drop(buffer);
            self.flush_batch(events).await;
        }
    }

    pub async fn flush(&self) {
        let mut buffer = self.buffer.lock().await;
        if buffer.is_empty() {
            return;
        }
        let events = std::mem::take(&mut *buffer);
        drop(buffer);
        self.flush_batch(events).await;
    }

    async fn flush_batch(&self, events: Vec<GameEvent>) {
        if events.is_empty() {
            return;
        }

        let mut chats = Vec::new();
        let mut damaged = Vec::new();
        let mut died = Vec::new();
        let mut connected = Vec::new();
        let mut deployables = Vec::new();
        let mut tick_rates = Vec::new();
        let mut matches = Vec::new();
        let mut vehicles = Vec::new();
        let mut vehicle_damages = Vec::new();
        let mut flies = Vec::new();

        for event in events {
            match event {
                GameEvent::Chat(e) => chats.push(e),
                GameEvent::Damaged(e) => damaged.push(e),
                GameEvent::Died(e) => died.push(e),
                GameEvent::Connected(e) => connected.push(e),
                GameEvent::Deployable(e) => deployables.push(e),
                GameEvent::TickRate(e) => tick_rates.push(e),
                GameEvent::Match(e) => matches.push(e),
                GameEvent::Vehicle(e) => vehicles.push(e),
                GameEvent::VehicleDamage(e) => vehicle_damages.push(e),
                GameEvent::Fly(e) => flies.push(e),
            }
        }

        let client = self.pool.client();
        let db = self.pool.database.clone();

        tokio::spawn(async move {
            let mut handles = Vec::new();

            if !chats.is_empty() {
                let c = client.clone();
                handles.push(tokio::spawn(async move {
                    let mut insert = c.insert::<ChatMessageEvent>(&format!("{}.player_chat_messages", db)).await?;
                    for e in chats { insert.write(&e).await?; }
                    insert.end().await?;
                    Ok::<(), clickhouse::error::Error>(())
                }));
            }
            if !damaged.is_empty() {
                let c = client.clone();
                handles.push(tokio::spawn(async move {
                    let mut insert = c.insert::<PlayerDamagedEvent>(&format!("{}.player_damaged_events", db)).await?;
                    for e in damaged { insert.write(&e).await?; }
                    insert.end().await?;
                    Ok::<(), clickhouse::error::Error>(())
                }));
            }
            if !died.is_empty() {
                let c = client.clone();
                handles.push(tokio::spawn(async move {
                    let mut insert = c.insert::<PlayerDiedEvent>(&format!("{}.player_died_events", db)).await?;
                    for e in died { insert.write(&e).await?; }
                    insert.end().await?;
                    Ok::<(), clickhouse::error::Error>(())
                }));
            }
            if !connected.is_empty() {
                let c = client.clone();
                handles.push(tokio::spawn(async move {
                    let mut insert = c.insert::<PlayerConnectedEvent>(&format!("{}.player_connected_events", db)).await?;
                    for e in connected { insert.write(&e).await?; }
                    insert.end().await?;
                    Ok::<(), clickhouse::error::Error>(())
                }));
            }
            if !deployables.is_empty() {
                let c = client.clone();
                handles.push(tokio::spawn(async move {
                    let mut insert = c.insert::<DeployableEvent>(&format!("{}.deployable_events", db)).await?;
                    for e in deployables { insert.write(&e).await?; }
                    insert.end().await?;
                    Ok::<(), clickhouse::error::Error>(())
                }));
            }
            if !tick_rates.is_empty() {
                let c = client.clone();
                handles.push(tokio::spawn(async move {
                    let mut insert = c.insert::<TickRateEvent>(&format!("{}.tick_rate_events", db)).await?;
                    for e in tick_rates { insert.write(&e).await?; }
                    insert.end().await?;
                    Ok::<(), clickhouse::error::Error>(())
                }));
            }
            if !matches.is_empty() {
                let c = client.clone();
                handles.push(tokio::spawn(async move {
                    let mut insert = c.insert::<MatchEvent>(&format!("{}.match_events", db)).await?;
                    for e in matches { insert.write(&e).await?; }
                    insert.end().await?;
                    Ok::<(), clickhouse::error::Error>(())
                }));
            }
            if !vehicles.is_empty() {
                let c = client.clone();
                handles.push(tokio::spawn(async move {
                    let mut insert = c.insert::<VehicleEvent>(&format!("{}.vehicle_events", db)).await?;
                    for e in vehicles { insert.write(&e).await?; }
                    insert.end().await?;
                    Ok::<(), clickhouse::error::Error>(())
                }));
            }
            if !vehicle_damages.is_empty() {
                let c = client.clone();
                handles.push(tokio::spawn(async move {
                    let mut insert = c.insert::<VehicleDamageEvent>(&format!("{}.vehicle_damage_events", db)).await?;
                    for e in vehicle_damages { insert.write(&e).await?; }
                    insert.end().await?;
                    Ok::<(), clickhouse::error::Error>(())
                }));
            }
            if !flies.is_empty() {
                let c = client.clone();
                handles.push(tokio::spawn(async move {
                    let mut insert = c.insert::<FlyEvent>(&format!("{}.fly_events", db)).await?;
                    for e in flies { insert.write(&e).await?; }
                    insert.end().await?;
                    Ok::<(), clickhouse::error::Error>(())
                }));
            }

            for handle in handles {
                if let Err(e) = handle.await {
                    tracing::error!("ClickHouse 批量写入任务失败: {}", e);
                }
            }
        });
    }

    pub async fn start_flush_task(&self) {
        if !self.enabled {
            return;
        }
        let ingest = self.clone();
        tokio::spawn(async move {
            let mut ticker = interval(FLUSH_INTERVAL);
            ticker.tick().await;
            loop {
                ticker.tick().await;
                ingest.flush().await;
            }
        });
    }
}

impl Clone for ClickHouseIngestService {
    fn clone(&self) -> Self {
        Self {
            pool: Arc::clone(&self.pool),
            buffer: Mutex::new(Vec::new()),
            enabled: self.enabled,
        }
    }
}