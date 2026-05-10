# 伤害通知服务增强设计

日期: 2026-05-10
状态: 已批准

## 概述

向现有 `damage_notify_service.rs` 统一服务中补充 4 项功能，对齐参考项目（squad-Super-Boy-main）的 `kill_broadcast` 和 `auto_tk_warn` 插件能力，同时保持"一个服务管全部"的架构优势。

## 新增功能

1. **击杀通知** — 玩家被击杀时通知攻击者和受害者
2. **消息模板化** — 所有通知消息支持 `{{attacker}}` `{{victim}}` `{{damage}}` `{{weapon}}` `{{seconds}}` `{{keyword}}` 变量，管理员后台可编辑
3. **道歉预窗口** — 玩家道歉成功后 20 秒内再次误伤免重复要求道歉
4. **三种发送模式** — `broadcast`（全服广播）/ `warning_all`（全服私发）/ `warning_related`（仅当事人）

## 架构决策

- **方案 A：原地扩展** — 在现有 `damage_notify_service.rs` 内增加功能，不拆分服务
- 理由：保持统一服务优势，文件增长至 ~700 行仍在可维护范围内，不需要修改 `main.rs` 服务启动逻辑

## 数据库变更

### 迁移 1: `032_expand_damage_notify_templates.sql`

`damage_notify_settings` 表新增字段：

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `message_mode` | VARCHAR(32) | `'warning_related'` | 发送模式 |
| `hit_layout` | TEXT | `'[命中] {{attacker}} 对 {{victim}} 造成了 {{damage}} 点伤害，使用 {{weapon}}'` | 伤害通知模板 |
| `kill_layout` | TEXT | `'[击杀] {{attacker}} 击杀了 {{victim}}，造成 {{damage}} 点伤害，使用 {{weapon}}'` | 击杀通知模板 |

### 迁移 2: `033_expand_tk_templates.sql`

`tk_settings` 表新增字段：

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `apology_pre_window_secs` | INTEGER | `20` | 道歉预窗口秒数 |
| `tk_attacker_msg` | TEXT | `'你对队友 {{victim}} 造成了友伤，请在 {{seconds}} 秒内输入 {{keyword}} 道歉'` | 攻击者消息模板 |
| `tk_victim_msg` | TEXT | `'你被队友 {{attacker}} 误伤了'` | 受害者消息模板 |
| `tk_broadcast_msg` | TEXT | `'[TK] {{attacker}} 误伤了 {{victim}}，请输入 {{keyword}} 道歉'` | TK 广播模板 |

## 服务层变更 (`damage_notify_service.rs`)

### 新增：模板渲染函数

```rust
fn render_template(template: &str, vars: &HashMap<&str, String>) -> String
```

- 替换所有 `{{key}}` 为对应值
- 未找到的变量渲染为 `"未知"`
- 模板为空返回空字符串（跳过发送）

### 新增：统一消息分发

```rust
async fn dispatch_message(
    pool: &PgPool,
    server_id: i32,
    message: &str,
    mode: &str,
    attacker_steam64: &str,
    victim_steam64: &str,
    server_states: &Arc<RwLock<HashMap<String, serde_json::Value>>>,
    rcon_pool: &RconPool,
)
```

| mode | 行为 |
|------|------|
| `broadcast` | `AdminBroadcast {message}` |
| `warning_all` | 遍历 server_states 中所有在线玩家，逐个 `AdminWarn {pid} {message}` |
| `warning_related` | 仅对 attacker + victim 发 `AdminWarn {pid} {message}` |

### 改造：`handle_enemy_damage`

- 从数据库读取 `hit_layout` + `message_mode`
- 用 `render_template` 渲染消息
- 用 `dispatch_message` 按模式发送

### 新增：`handle_kill_notify`

- 触发条件：`KillEvent.event_type == "death"` 且 `notify_kill == true` 且 `same_team == false`
- 从数据库读取 `kill_layout` + `message_mode`
- 用 `render_template` 渲染，`dispatch_message` 发送

### 改造：`handle_teamkill`

- 从 `tk_settings` 读取 `tk_attacker_msg`、`tk_victim_msg`、`tk_broadcast_msg` 模板
- 攻击者消息：`render_template(tk_attacker_msg)` → `AdminWarn` 发给攻击者
- 受害者消息：`render_template(tk_victim_msg)` → `AdminWarn` 发给受害者（新增）
- 广播消息：`render_template(tk_broadcast_msg)` → `AdminBroadcast`

### 新增：道歉预窗口

`PlayerTkState` 新增字段：

```rust
last_apology_at: Option<Instant>,
```

逻辑：
1. `mark_apologized` 时记录 `last_apology_at = Instant::now()`
2. `handle_teamkill` 开头检查：如果 `last_apology_at` 存在且距今 < `apology_pre_window_secs` → 跳过道歉要求（仍发广播和警告，但不启动踢出计时器）
3. `apology_pre_window_secs` 从 `tk_settings` 读取，默认 20

### 事件分流

```
KillEvent { event_type: "damage", same_team: true  } → handle_teamkill
KillEvent { event_type: "damage", same_team: false } → handle_enemy_damage
KillEvent { event_type: "death",  same_team: false } → handle_kill_notify
KillEvent { event_type: "death",  same_team: true  } → 仅日志（TK 在 damage 阶段已处理）
KillEvent { event_type: "wound",  same_team: false } → handle_kill_notify（击倒通知）
KillEvent { event_type: "wound",  same_team: true  } → handle_teamkill（同 damage）
```

## Model 层变更

### `models/damage_notify_settings.rs`

新增字段：`message_mode: String`、`hit_layout: String`、`kill_layout: String`

`UpdateDamageNotifyRequest` 对应新增 `Option` 字段。

### `models/tk_settings.rs`

新增字段：`apology_pre_window_secs: i32`、`tk_attacker_msg: Option<String>`、`tk_victim_msg: Option<String>`、`tk_broadcast_msg: Option<String>`

`UpdateTkSettingsRequest` 对应新增 `Option` 字段。

## Repository 层变更

- `damage_notify_repo.rs`：SQL 查询补上 `message_mode`、`hit_layout`、`kill_layout`
- `tk_settings_repo.rs`：SQL 查询补上 `apology_pre_window_secs`、`tk_attacker_msg`、`tk_victim_msg`、`tk_broadcast_msg`

## API 层

无结构变更。现有 `get/update` 接口自动携带新字段（struct 序列化）。

## 改动文件清单

| 文件 | 操作 |
|------|------|
| `migrations/032_expand_damage_notify_templates.sql` | 新建 |
| `migrations/033_expand_tk_templates.sql` | 新建 |
| `src/models/damage_notify_settings.rs` | 修改 |
| `src/models/tk_settings.rs` | 修改 |
| `src/repositories/damage_notify_repo.rs` | 修改 |
| `src/repositories/tk_settings_repo.rs` | 修改 |
| `src/services/damage_notify_service.rs` | 主要修改 |

## 不改动

- `main.rs` 服务启动逻辑
- API 路由结构
- `squad_log_parser.rs`（已有所需字段）
- 前端（本次仅后端，前端如需可后续迭代）
