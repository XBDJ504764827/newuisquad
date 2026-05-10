# 伤害通知服务增强 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 向 `damage_notify_service.rs` 添加击杀通知、消息模板化、道歉预窗口和三种发送模式。

**Architecture:** 在现有统一服务内原地扩展，新增 `render_template` 和 `dispatch_message` 函数作为核心基础设施，然后改造现有 handler 并新增 `handle_kill_notify`。数据库通过两个 ALTER TABLE 迁移新增字段。

**Tech Stack:** Rust, tokio, sqlx (PostgreSQL), RCON protocol

---

## File Structure

| 文件 | 操作 | 职责 |
|------|------|------|
| `backend/migrations/038_expand_damage_notify_templates.sql` | 新建 | damage_notify_settings 表加 message_mode, hit_layout, kill_layout |
| `backend/migrations/039_expand_tk_templates.sql` | 新建 | tk_settings 表加 apology_pre_window_secs, tk_attacker_msg, tk_victim_msg, tk_broadcast_msg |
| `backend/src/models/damage_notify_settings.rs` | 修改 | 结构体加新字段 |
| `backend/src/models/tk_settings.rs` | 修改 | 结构体加新字段 |
| `backend/src/repositories/damage_notify_repo.rs` | 修改 | SQL 补新列 |
| `backend/src/repositories/tk_settings_repo.rs` | 修改 | SQL 补新列 |
| `backend/src/services/damage_notify_service.rs` | 修改 | 核心逻辑增强 |

---

### Task 1: 数据库迁移 — damage_notify_settings 模板字段

**Files:**
- Create: `backend/migrations/038_expand_damage_notify_templates.sql`

- [ ] **Step 1: 创建迁移文件**

```sql
-- 伤害通知: 添加消息模板和发送模式
ALTER TABLE damage_notify_settings
    ADD COLUMN IF NOT EXISTS message_mode VARCHAR(32) NOT NULL DEFAULT 'warning_related',
    ADD COLUMN IF NOT EXISTS hit_layout TEXT NOT NULL DEFAULT '[命中] {{attacker}} 对 {{victim}} 造成了 {{damage}} 点伤害，使用 {{weapon}}',
    ADD COLUMN IF NOT EXISTS kill_layout TEXT NOT NULL DEFAULT '[击杀] {{attacker}} 击杀了 {{victim}}，造成 {{damage}} 点伤害，使用 {{weapon}}';
```

- [ ] **Step 2: 验证迁移语法**

Run: `cd /home/xbdj/newuisquad/backend && cargo build 2>&1 | head -20`
Expected: 编译成功（迁移文件不影响编译，仅确认项目仍可构建）

- [ ] **Step 3: Commit**

```bash
git add backend/migrations/038_expand_damage_notify_templates.sql
git commit -m "db: add message_mode, hit_layout, kill_layout to damage_notify_settings"
```

---

### Task 2: 数据库迁移 — tk_settings 模板字段

**Files:**
- Create: `backend/migrations/039_expand_tk_templates.sql`

- [ ] **Step 1: 创建迁移文件**

```sql
-- TK 设置: 添加道歉预窗口和消息模板
ALTER TABLE tk_settings
    ADD COLUMN IF NOT EXISTS apology_pre_window_secs INTEGER NOT NULL DEFAULT 20,
    ADD COLUMN IF NOT EXISTS tk_attacker_msg TEXT NOT NULL DEFAULT '你对队友 {{victim}} 造成了友伤，请在 {{seconds}} 秒内输入 {{keyword}} 道歉',
    ADD COLUMN IF NOT EXISTS tk_victim_msg TEXT NOT NULL DEFAULT '你被队友 {{attacker}} 误伤了',
    ADD COLUMN IF NOT EXISTS tk_broadcast_msg TEXT NOT NULL DEFAULT '[TK] {{attacker}} 误伤了 {{victim}}，请输入 {{keyword}} 道歉';
```

- [ ] **Step 2: Commit**

```bash
git add backend/migrations/039_expand_tk_templates.sql
git commit -m "db: add apology_pre_window_secs, tk message templates to tk_settings"
```

---

### Task 3: Model 层 — DamageNotifySettings 扩展

**Files:**
- Modify: `backend/src/models/damage_notify_settings.rs`

- [ ] **Step 1: 更新 DamageNotifySettings 结构体**

将文件内容替换为：

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DamageNotifySettings {
    pub id: i32,
    pub server_id: i32,
    pub enabled: bool,
    pub notify_kill: bool,
    pub notify_damage: bool,
    pub message_mode: String,
    pub hit_layout: String,
    pub kill_layout: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDamageNotifyRequest {
    pub enabled: Option<bool>,
    pub notify_kill: Option<bool>,
    pub notify_damage: Option<bool>,
    pub message_mode: Option<String>,
    pub hit_layout: Option<String>,
    pub kill_layout: Option<String>,
}
```

- [ ] **Step 2: 验证编译**

Run: `cd /home/xbdj/newuisquad/backend && cargo check 2>&1 | tail -5`
Expected: 可能有 repo 层的编译错误（SQL 列不匹配），Task 5 会修复

- [ ] **Step 3: Commit**

```bash
git add backend/src/models/damage_notify_settings.rs
git commit -m "model: add message_mode, hit_layout, kill_layout to DamageNotifySettings"
```

---

### Task 4: Model 层 — TkSettings 扩展

**Files:**
- Modify: `backend/src/models/tk_settings.rs`

- [ ] **Step 1: 更新 TkSettings 结构体**

将文件内容替换为：

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TkSettings {
    pub id: i32,
    pub server_id: i32,
    pub enabled: bool,
    pub max_team_kills: i32,
    pub apology_time_minutes: i32,
    pub apology_keyword: String,
    pub notification_message: Option<String>,
    pub tk_broadcast_message: Option<String>,
    pub apology_pre_window_secs: i32,
    pub tk_attacker_msg: String,
    pub tk_victim_msg: String,
    pub tk_broadcast_msg: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTkSettingsRequest {
    pub enabled: Option<bool>,
    pub max_team_kills: Option<i32>,
    pub apology_time_minutes: Option<i32>,
    pub apology_keyword: Option<String>,
    pub notification_message: Option<String>,
    pub tk_broadcast_message: Option<String>,
    pub apology_pre_window_secs: Option<i32>,
    pub tk_attacker_msg: Option<String>,
    pub tk_victim_msg: Option<String>,
    pub tk_broadcast_msg: Option<String>,
}
```

- [ ] **Step 2: Commit**

```bash
git add backend/src/models/tk_settings.rs
git commit -m "model: add apology_pre_window_secs and tk message templates to TkSettings"
```

---

### Task 5: Repository 层 — damage_notify_repo 更新

**Files:**
- Modify: `backend/src/repositories/damage_notify_repo.rs`

- [ ] **Step 1: 更新 SQL 查询以包含新字段**

将文件内容替换为：

```rust
use sqlx::PgPool;
use crate::models::damage_notify_settings::{DamageNotifySettings, UpdateDamageNotifyRequest};

pub async fn get_or_create(pool: &PgPool, server_id: i32) -> Result<DamageNotifySettings, sqlx::Error> {
    let existing = sqlx::query_as::<_, DamageNotifySettings>(
        "SELECT id, server_id, enabled, notify_kill, notify_damage, message_mode, hit_layout, kill_layout, updated_at \
         FROM damage_notify_settings WHERE server_id = $1"
    ).bind(server_id).fetch_optional(pool).await?;
    if let Some(s) = existing { return Ok(s); }
    sqlx::query_as::<_, DamageNotifySettings>(
        "INSERT INTO damage_notify_settings (server_id) VALUES ($1) \
         RETURNING id, server_id, enabled, notify_kill, notify_damage, message_mode, hit_layout, kill_layout, updated_at"
    ).bind(server_id).fetch_one(pool).await
}

pub async fn update(pool: &PgPool, server_id: i32, req: &UpdateDamageNotifyRequest) -> Result<DamageNotifySettings, sqlx::Error> {
    let c = get_or_create(pool, server_id).await?;
    sqlx::query_as::<_, DamageNotifySettings>(
        "UPDATE damage_notify_settings SET \
         enabled=$1, notify_kill=$2, notify_damage=$3, message_mode=$4, hit_layout=$5, kill_layout=$6, updated_at=NOW() \
         WHERE server_id=$7 \
         RETURNING id, server_id, enabled, notify_kill, notify_damage, message_mode, hit_layout, kill_layout, updated_at"
    )
    .bind(req.enabled.unwrap_or(c.enabled))
    .bind(req.notify_kill.unwrap_or(c.notify_kill))
    .bind(req.notify_damage.unwrap_or(c.notify_damage))
    .bind(req.message_mode.as_deref().unwrap_or(&c.message_mode))
    .bind(req.hit_layout.as_deref().unwrap_or(&c.hit_layout))
    .bind(req.kill_layout.as_deref().unwrap_or(&c.kill_layout))
    .bind(server_id)
    .fetch_one(pool).await
}
```

- [ ] **Step 2: Commit**

```bash
git add backend/src/repositories/damage_notify_repo.rs
git commit -m "repo: update damage_notify_repo SQL for new template fields"
```

---

### Task 6: Repository 层 — tk_settings_repo 更新

**Files:**
- Modify: `backend/src/repositories/tk_settings_repo.rs`

- [ ] **Step 1: 更新 SQL 查询以包含新字段**

将文件内容替换为：

```rust
use sqlx::PgPool;
use crate::models::tk_settings::{TkSettings, UpdateTkSettingsRequest};

pub async fn get_or_create(pool: &PgPool, server_id: i32) -> Result<TkSettings, sqlx::Error> {
    let existing = sqlx::query_as::<_, TkSettings>("SELECT * FROM tk_settings WHERE server_id = $1")
        .bind(server_id)
        .fetch_optional(pool)
        .await?;
    if let Some(s) = existing { return Ok(s); }
    sqlx::query_as::<_, TkSettings>(
        "INSERT INTO tk_settings (server_id) VALUES ($1) RETURNING *"
    ).bind(server_id).fetch_one(pool).await
}

pub async fn update(pool: &PgPool, server_id: i32, req: &UpdateTkSettingsRequest) -> Result<TkSettings, sqlx::Error> {
    let current = get_or_create(pool, server_id).await?;
    sqlx::query_as::<_, TkSettings>(
        "UPDATE tk_settings SET \
         enabled=$1, max_team_kills=$2, apology_time_minutes=$3, apology_keyword=$4, \
         notification_message=$5, tk_broadcast_message=$6, \
         apology_pre_window_secs=$7, tk_attacker_msg=$8, tk_victim_msg=$9, tk_broadcast_msg=$10, \
         updated_at=NOW() \
         WHERE server_id=$11 RETURNING *"
    )
    .bind(req.enabled.unwrap_or(current.enabled))
    .bind(req.max_team_kills.unwrap_or(current.max_team_kills))
    .bind(req.apology_time_minutes.unwrap_or(current.apology_time_minutes))
    .bind(req.apology_keyword.as_deref().unwrap_or(&current.apology_keyword))
    .bind(req.notification_message.as_deref().unwrap_or(current.notification_message.as_deref().unwrap_or("")))
    .bind(req.tk_broadcast_message.as_deref().unwrap_or(current.tk_broadcast_message.as_deref().unwrap_or("")))
    .bind(req.apology_pre_window_secs.unwrap_or(current.apology_pre_window_secs))
    .bind(req.tk_attacker_msg.as_deref().unwrap_or(&current.tk_attacker_msg))
    .bind(req.tk_victim_msg.as_deref().unwrap_or(&current.tk_victim_msg))
    .bind(req.tk_broadcast_msg.as_deref().unwrap_or(&current.tk_broadcast_msg))
    .bind(server_id)
    .fetch_one(pool)
    .await
}
```

- [ ] **Step 2: 验证编译**

Run: `cd /home/xbdj/newuisquad/backend && cargo check 2>&1 | tail -10`
Expected: model + repo 层编译通过（service 层可能仍有警告）

- [ ] **Step 3: Commit**

```bash
git add backend/src/repositories/tk_settings_repo.rs
git commit -m "repo: update tk_settings_repo SQL for new template and pre-window fields"
```

---

### Task 7: Service 层 — render_template 函数

**Files:**
- Modify: `backend/src/services/damage_notify_service.rs`

- [ ] **Step 1: 在文件顶部 use 区域后（`use crate::services::system_log;` 之后）添加 render_template 函数**

在 `// ═══ API 层使用的 CRUD 函数 ═══` 注释之前插入：

```rust
// ═══ 模板渲染 ═══

/// 将模板中的 {{key}} 替换为 vars 中对应的值。
/// 未匹配的变量替换为 "未知"，模板为空则返回空字符串。
fn render_template(template: &str, vars: &[(&str, &str)]) -> String {
    let template = template.trim();
    if template.is_empty() {
        return String::new();
    }
    let mut result = template.to_string();
    for (key, value) in vars {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }
    // 替换剩余未匹配的 {{xxx}} 为 "未知"
    while let Some(start) = result.find("{{") {
        if let Some(end) = result[start..].find("}}") {
            result.replace_range(start..start + end + 2, "未知");
        } else {
            break;
        }
    }
    result
}
```

- [ ] **Step 2: 验证编译**

Run: `cd /home/xbdj/newuisquad/backend && cargo check 2>&1 | tail -5`
Expected: 编译通过（函数未使用会有 warning，可接受）

- [ ] **Step 3: Commit**

```bash
git add backend/src/services/damage_notify_service.rs
git commit -m "feat(damage_notify): add render_template utility function"
```

---

### Task 8: Service 层 — dispatch_message 函数

**Files:**
- Modify: `backend/src/services/damage_notify_service.rs`

- [ ] **Step 1: 在 render_template 函数之后添加 dispatch_message 函数**

```rust
// ═══ 统一消息分发 ═══

/// 根据 mode 选择发送方式：
/// - "broadcast": AdminBroadcast 全服广播
/// - "warning_all": 给所有在线玩家发 AdminWarn
/// - "warning_related": 仅给 attacker + victim 发 AdminWarn（默认）
async fn dispatch_message(
    pool: &PgPool,
    server_id: i32,
    message: &str,
    mode: &str,
    attacker_steam64: &str,
    victim_name: &str,
    server_states: &Arc<tokio::sync::RwLock<HashMap<String, serde_json::Value>>>,
    rcon_pool: &RconPool,
) {
    if message.is_empty() { return; }

    match mode {
        "broadcast" => {
            let cmd = format!("AdminBroadcast {}", message);
            send_rcon_cmd(pool, server_id, &cmd, rcon_pool).await;
        }
        "warning_all" => {
            let states = server_states.read().await;
            if let Some(state) = states.get(&server_id.to_string()) {
                if let Some(players) = state.get("players").and_then(|p| p.as_array()) {
                    for p in players {
                        if let Some(pid) = p.get("player_id").and_then(|id| id.as_i64()) {
                            let cmd = format!("AdminWarn {} {}", pid, message);
                            send_rcon_cmd(pool, server_id, &cmd, rcon_pool).await;
                        }
                    }
                }
            }
        }
        _ => {
            // warning_related: 仅发给攻击者和受害者
            let states = server_states.read().await;
            let attacker_pid = find_player_id_by_steam(&states, server_id, attacker_steam64);
            let victim_pid = find_player_id(&states, server_id, victim_name);
            drop(states);

            if let Some(pid) = attacker_pid {
                let cmd = format!("AdminWarn {} {}", pid, message);
                send_rcon_cmd(pool, server_id, &cmd, rcon_pool).await;
            }
            if let Some(pid) = victim_pid {
                let cmd = format!("AdminWarn {} {}", pid, message);
                send_rcon_cmd(pool, server_id, &cmd, rcon_pool).await;
            }
        }
    }
}
```

- [ ] **Step 2: 验证编译**

Run: `cd /home/xbdj/newuisquad/backend && cargo check 2>&1 | tail -5`
Expected: 编译通过

- [ ] **Step 3: Commit**

```bash
git add backend/src/services/damage_notify_service.rs
git commit -m "feat(damage_notify): add dispatch_message with broadcast/warning_all/warning_related modes"
```

---

### Task 9: Service 层 — 改造 handle_enemy_damage 使用模板和分发

**Files:**
- Modify: `backend/src/services/damage_notify_service.rs`

- [ ] **Step 1: 替换 handle_enemy_damage 函数**

将现有的 `handle_enemy_damage` 函数替换为：

```rust
/// 处理敌方伤害通知
async fn handle_enemy_damage(
    pool: &PgPool, server_id: i32,
    attacker: &str, attacker_steam64: &str,
    victim: &str, damage: f64, weapon: &str,
    server_states: &Arc<tokio::sync::RwLock<HashMap<String, serde_json::Value>>>,
    rcon_pool: &RconPool,
) {
    // 读取配置：notify_damage 开关 + hit_layout + message_mode
    let config = match sqlx::query_as::<_, (bool, String, String)>(
        "SELECT notify_damage, hit_layout, message_mode FROM damage_notify_settings WHERE server_id = $1"
    ).bind(server_id).fetch_optional(pool).await {
        Ok(Some(c)) => c,
        _ => return,
    };
    let (notify_damage, hit_layout, message_mode) = config;
    if !notify_damage { return; }

    let damage_str = format!("{:.0}", damage);
    let message = render_template(&hit_layout, &[
        ("attacker", attacker),
        ("victim", victim),
        ("damage", &damage_str),
        ("weapon", weapon),
    ]);

    dispatch_message(pool, server_id, &message, &message_mode, attacker_steam64, victim, server_states, rcon_pool).await;
}
```

- [ ] **Step 2: 更新调用点 — 在 start_damage_notify 的事件循环中传入新参数**

在事件循环的 `KillEvent` 匹配中，需要解构出 `weapon` 和 `event_type` 字段。将现有的：

```rust
if let ParsedEvent::KillEvent {
    ref attacker_name, ref attacker_steam64,
    ref victim_name, damage, ..
} = event {
```

替换为：

```rust
if let ParsedEvent::KillEvent {
    ref attacker_name, ref attacker_steam64,
    ref victim_name, damage,
    ref weapon, ref event_type, ..
} = event {
```

并将 `Some(false)` 分支的 `handle_enemy_damage` 调用更新为：

```rust
Some(false) => {
    if event_type == "damage" {
        // 敌方伤害
        handle_enemy_damage(
            &pool, server_id,
            attacker, attacker_steam64, victim, damage, weapon,
            &server_states, &rcon_pool,
        ).await;
    } else if event_type == "death" || event_type == "wound" {
        // 击杀/击倒通知
        handle_kill_notify(
            &pool, server_id,
            attacker, attacker_steam64, victim, damage, weapon,
            &server_states, &rcon_pool,
        ).await;
    }
}
```

- [ ] **Step 3: 添加 handle_kill_notify 占位函数（防止编译错误）**

在 `handle_enemy_damage` 之后添加占位：

```rust
/// 处理击杀/击倒通知（Task 10 完善）
async fn handle_kill_notify(
    _pool: &PgPool, _server_id: i32,
    _attacker: &str, _attacker_steam64: &str,
    _victim: &str, _damage: f64, _weapon: &str,
    _server_states: &Arc<tokio::sync::RwLock<HashMap<String, serde_json::Value>>>,
    _rcon_pool: &RconPool,
) {
    // 占位，Task 10 实现
}
```

- [ ] **Step 4: 同时更新 Some(true) 分支传入 weapon**

将 `Some(true)` 分支的 `handle_teamkill` 调用更新为（保持编译通过，函数签名在 Task 11 才改，先兼容）：

因为 `handle_teamkill` 的签名还没变，这里先不改。event_type == "wound" 同队的情况也走 handle_teamkill，所以将 `Some(true)` 分支改为：

```rust
Some(true) => {
    if event_type == "damage" || event_type == "wound" {
        // 友方误伤
        handle_teamkill(
            &pool, &tracker, &server_states, server_id,
            attacker, attacker_steam64, victim, damage,
            attacker_pid, &rcon_pool,
        ).await;
    }
    // event_type == "death" 且 same_team: TK击杀在damage阶段已处理，跳过
}
```

- [ ] **Step 5: 验证编译**

Run: `cd /home/xbdj/newuisquad/backend && cargo check 2>&1 | tail -10`
Expected: 编译通过（可能有 unused 警告）

- [ ] **Step 6: Commit**

```bash
git add backend/src/services/damage_notify_service.rs
git commit -m "feat(damage_notify): refactor handle_enemy_damage to use templates and dispatch_message"
```

---

### Task 10: Service 层 — 实现 handle_kill_notify

**Files:**
- Modify: `backend/src/services/damage_notify_service.rs`

- [ ] **Step 1: 替换 handle_kill_notify 占位为完整实现**

```rust
/// 处理击杀/击倒通知（敌方）
async fn handle_kill_notify(
    pool: &PgPool, server_id: i32,
    attacker: &str, attacker_steam64: &str,
    victim: &str, damage: f64, weapon: &str,
    server_states: &Arc<tokio::sync::RwLock<HashMap<String, serde_json::Value>>>,
    rcon_pool: &RconPool,
) {
    // 读取配置：notify_kill 开关 + kill_layout + message_mode
    let config = match sqlx::query_as::<_, (bool, String, String)>(
        "SELECT notify_kill, kill_layout, message_mode FROM damage_notify_settings WHERE server_id = $1"
    ).bind(server_id).fetch_optional(pool).await {
        Ok(Some(c)) => c,
        _ => return,
    };
    let (notify_kill, kill_layout, message_mode) = config;
    if !notify_kill { return; }

    let damage_str = format!("{:.0}", damage);
    let message = render_template(&kill_layout, &[
        ("attacker", attacker),
        ("victim", victim),
        ("damage", &damage_str),
        ("weapon", weapon),
    ]);

    dispatch_message(pool, server_id, &message, &message_mode, attacker_steam64, victim, server_states, rcon_pool).await;
}
```

- [ ] **Step 2: 验证编译**

Run: `cd /home/xbdj/newuisquad/backend && cargo check 2>&1 | tail -5`
Expected: 编译通过

- [ ] **Step 3: Commit**

```bash
git add backend/src/services/damage_notify_service.rs
git commit -m "feat(damage_notify): implement handle_kill_notify for death/wound events"
```

---

### Task 11: Service 层 — 改造 handle_teamkill 使用模板

**Files:**
- Modify: `backend/src/services/damage_notify_service.rs`

- [ ] **Step 1: 替换 handle_teamkill 函数**

将现有的 `handle_teamkill` 函数替换为：

```rust
/// 处理友方误伤 — 使用模板发送消息并启动道歉倒计时
async fn handle_teamkill(
    pool: &PgPool,
    tracker: &Arc<RwLock<TkTracker>>,
    server_states: &Arc<tokio::sync::RwLock<HashMap<String, serde_json::Value>>>,
    server_id: i32,
    attacker: &str,
    attacker_steam64: &str,
    victim: &str,
    damage: f64,
    attacker_pid: Option<i32>,
    rcon_pool: &RconPool,
) {
    // 查询 TK 设置
    let tk_config = match sqlx::query_as::<_, (bool, i32, String, i32, String, String, String)>(
        "SELECT enabled, apology_time_minutes, apology_keyword, apology_pre_window_secs, \
         tk_attacker_msg, tk_victim_msg, tk_broadcast_msg \
         FROM tk_settings WHERE server_id = $1"
    ).bind(server_id).fetch_optional(pool).await {
        Ok(Some(c)) => c,
        _ => return,
    };
    let (tk_enabled, apology_minutes, apology_keyword, pre_window_secs,
         tk_attacker_msg, tk_victim_msg, tk_broadcast_msg) = tk_config;
    if !tk_enabled { return; }

    let pid_str = attacker_pid.map(|id| id.to_string()).unwrap_or_else(|| attacker.to_string());
    let seconds_str = (apology_minutes * 60).to_string();
    let damage_str = format!("{:.0}", damage);

    // 模板变量
    let vars: Vec<(&str, &str)> = vec![
        ("attacker", attacker),
        ("victim", victim),
        ("damage", &damage_str),
        ("seconds", &seconds_str),
        ("keyword", &apology_keyword),
    ];

    // 1. 广播消息
    let broadcast_msg = render_template(&tk_broadcast_msg, &vars);
    if !broadcast_msg.is_empty() {
        send_rcon_cmd(pool, server_id, &format!("AdminBroadcast {}", broadcast_msg), rcon_pool).await;
    }

    // 2. 攻击者私发警告
    let attacker_msg = render_template(&tk_attacker_msg, &vars);
    if !attacker_msg.is_empty() {
        let cmd = format!("AdminWarn {} {}", pid_str, attacker_msg);
        send_rcon_cmd(pool, server_id, &cmd, rcon_pool).await;
    }

    // 3. 受害者私发警告（新增）
    let victim_msg = render_template(&tk_victim_msg, &vars);
    if !victim_msg.is_empty() {
        let states = server_states.read().await;
        let victim_pid = find_player_id(&states, server_id, victim);
        drop(states);
        if let Some(vpid) = victim_pid {
            let cmd = format!("AdminWarn {} {}", vpid, victim_msg);
            send_rcon_cmd(pool, server_id, &cmd, rcon_pool).await;
        }
    }

    // 4. 道歉预窗口检查
    let skip_apology = {
        let t = tracker.read().await;
        if let Some(entry) = t.players.get(&(server_id, attacker_steam64.to_string())) {
            if let Some(last_apology) = entry.last_apology_at {
                last_apology.elapsed().as_secs() < pre_window_secs as u64
            } else {
                false
            }
        } else {
            false
        }
    };

    if skip_apology {
        tracing::info!(server_id, player = %attacker, victim = %victim, "道歉预窗口内，跳过踢出计时器");
        return;
    }

    // 5. 更新 TK 计数并重置道歉状态
    let (tk_count, timer_gen) = {
        let mut t = tracker.write().await;
        t.record_tk(server_id, attacker_steam64)
    };
    tracing::info!(server_id, player = %attacker, tk_count, victim = %victim, damage, timer_gen, "误伤事件");

    // 6. 启动道歉倒计时
    let deadline = Instant::now() + Duration::from_secs((apology_minutes as u64) * 60);
    tracker.write().await.set_apology_deadline(server_id, attacker_steam64, deadline);

    let tracker_clone = tracker.clone();
    let pool_clone = pool.clone();
    let server_states_clone = server_states.clone();
    let attacker_id = attacker_steam64.to_string();
    let attacker_name = attacker.to_string();
    let apology_kw = apology_keyword.clone();
    let rcon_pool_clone = rcon_pool.clone();

    tokio::spawn(async move {
        sleep(Duration::from_secs((apology_minutes as u64) * 60)).await;

        let t = tracker_clone.read().await;
        if t.is_apologized(server_id, &attacker_id)
            || t.is_kicked(server_id, &attacker_id)
            || !t.is_timer_current(server_id, &attacker_id, timer_gen)
        {
            return;
        }
        drop(t);

        let kick_pid = {
            let states = server_states_clone.read().await;
            find_player_id_by_steam(&states, server_id, &attacker_id)
                .or_else(|| find_player_id(&states, server_id, &attacker_name))
        };

        let kick_reason = format!(
            "您因误伤队友后未在{}分钟内输入{}道歉，已被踢出服务器",
            apology_minutes, apology_kw
        );

        if let Some(pid) = kick_pid {
            let kick_cmd = format!("AdminKickById {} {}", pid, kick_reason);
            send_rcon_cmd(&pool_clone, server_id, &kick_cmd, &rcon_pool_clone).await;
            tracing::info!(server_id, player = %attacker_name, player_id = pid, "玩家因未道歉被踢出 (AdminKickById)");
        } else {
            let kick_cmd = format!("AdminKick {} {}", attacker_name, kick_reason);
            send_rcon_cmd(&pool_clone, server_id, &kick_cmd, &rcon_pool_clone).await;
            tracing::warn!(server_id, player = %attacker_name, "无法获取 PlayerID，使用 AdminKick 名称踢出");
        }

        tracker_clone.write().await.mark_kicked(server_id, &attacker_id);
    });
}
```

- [ ] **Step 2: 验证编译**

Run: `cd /home/xbdj/newuisquad/backend && cargo check 2>&1 | tail -10`
Expected: 可能报错 `PlayerTkState` 没有 `last_apology_at` 字段 — Task 12 修复

- [ ] **Step 3: Commit**

```bash
git add backend/src/services/damage_notify_service.rs
git commit -m "feat(damage_notify): refactor handle_teamkill to use templates and victim warning"
```

---

### Task 12: Service 层 — 道歉预窗口（TkTracker 扩展）

**Files:**
- Modify: `backend/src/services/damage_notify_service.rs`

- [ ] **Step 1: 在 PlayerTkState 结构体中添加 last_apology_at 字段**

将 `PlayerTkState` 的定义：

```rust
#[derive(Debug, Clone)]
struct PlayerTkState {
    tk_count: u32,
    apology_deadline: Option<Instant>,
    apologized: bool,
    kicked: bool,
    timer_gen: u64,
}
```

替换为：

```rust
#[derive(Debug, Clone)]
struct PlayerTkState {
    tk_count: u32,
    apology_deadline: Option<Instant>,
    apologized: bool,
    kicked: bool,
    timer_gen: u64,
    last_apology_at: Option<Instant>,
}
```

- [ ] **Step 2: 更新 record_tk 中的初始化**

在 `record_tk` 方法中 `or_insert_with` 的闭包里添加 `last_apology_at: None`：

```rust
fn record_tk(&mut self, server_id: i32, steam_id: &str) -> (u32, u64) {
    let key = (server_id, steam_id.to_string());
    let entry = self.players.entry(key.clone()).or_insert_with(|| PlayerTkState {
        tk_count: 0, apology_deadline: None, apologized: false, kicked: false, timer_gen: 0,
        last_apology_at: None,
    });
    if entry.kicked { return (entry.tk_count, entry.timer_gen); }
    entry.tk_count += 1;
    entry.apology_deadline = None;
    entry.apologized = false;
    entry.timer_gen += 1;
    (entry.tk_count, entry.timer_gen)
}
```

- [ ] **Step 3: 更新 mark_apologized 记录道歉时间**

将 `mark_apologized` 方法改为：

```rust
fn mark_apologized(&mut self, server_id: i32, steam_id: &str) -> bool {
    let key = (server_id, steam_id.to_string());
    if let Some(entry) = self.players.get_mut(&key) {
        if entry.apology_deadline.is_some() && !entry.kicked {
            entry.apologized = true;
            entry.apology_deadline = None;
            entry.last_apology_at = Some(Instant::now());
            return true;
        }
    }
    false
}
```

- [ ] **Step 4: 验证编译**

Run: `cd /home/xbdj/newuisquad/backend && cargo check 2>&1 | tail -5`
Expected: 编译通过

- [ ] **Step 5: Commit**

```bash
git add backend/src/services/damage_notify_service.rs
git commit -m "feat(damage_notify): add apology pre-window via last_apology_at in TkTracker"
```

---

### Task 13: 最终编译验证和清理

**Files:**
- Modify: `backend/src/services/damage_notify_service.rs` (清理 unused imports/warnings)

- [ ] **Step 1: 完整构建验证**

Run: `cd /home/xbdj/newuisquad/backend && cargo build 2>&1`
Expected: 构建成功，无 error

- [ ] **Step 2: 清理 unused warnings**

检查输出中的 `unused` 警告，移除不再使用的旧代码（如旧的 `attacker_pid` 在 `handle_enemy_damage` 中不再需要等）。

具体可能需要的清理：
- `handle_enemy_damage` 的旧签名参数 `attacker_pid: Option<i32>` 已被移除
- 确保 `start_damage_notify` 中不再向 `handle_enemy_damage` 传 `attacker_pid`

- [ ] **Step 3: 验证无 error**

Run: `cd /home/xbdj/newuisquad/backend && cargo check 2>&1 | grep -i error`
Expected: 无输出（无错误）

- [ ] **Step 4: Commit**

```bash
git add backend/src/services/damage_notify_service.rs
git commit -m "chore(damage_notify): cleanup unused code and fix warnings"
```

---

## 验证清单

完成所有 Task 后，确认：

1. `cargo build` 无 error
2. 两个迁移文件存在于 `backend/migrations/`
3. `damage_notify_settings` 表有 `message_mode`、`hit_layout`、`kill_layout` 字段
4. `tk_settings` 表有 `apology_pre_window_secs`、`tk_attacker_msg`、`tk_victim_msg`、`tk_broadcast_msg` 字段
5. 事件分流覆盖：damage/wound/death × same_team/enemy
6. 道歉预窗口：`mark_apologized` 记录时间，`handle_teamkill` 检查窗口
7. 三种发送模式在 `dispatch_message` 中实现
