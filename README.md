# Squad 游戏服务器管理平台

一个全栈 **Squad** 游戏服务器管理平台，提供 Web 管理控制台、RCON 远程命令、实时玩家追踪、日志监控、自动化工作流等功能。

---

## 架构概览

```
┌─────────────────────────┐
│   admin-console (前端)   │  Next.js 16 + React 19 + Tailwind CSS 4
│   静态导出 → Nginx 代理  │
└──────────┬──────────────┘
           │ HTTP REST API (Bearer Token 认证)
┌──────────▼──────────────┐
│     backend (后端)       │  Rust + Axum 0.8 + SQLx + PostgreSQL
│   端口: 8000             │
└──┬──────────┬───────────┘
   │          │ WebSocket (WSS)
   │   ┌──────▼───────────┐
   │   │  agent (游戏端)   │  Rust + tokio-tungstenite
   │   │  部署在 Squad 服  │
   │   └──────┬───────────┘
   │          │ RCON
   ▼          ▼
┌─────────────────────────┐
│    Squad 游戏服务器       │
└─────────────────────────┘
```

| 组件 | 技术栈 | 说明 |
|------|--------|------|
| **backend/** | Rust + Axum 0.8 + SQLx 0.8 + PostgreSQL | REST API 后端，约 106 个 API 端点 |
| **agent/** | Rust + WebSocket (tokio-tungstenite) | 部署在游戏服务器，上报日志和状态 |
| **admin-console/** | Next.js 16 + React 19 + Tailwind CSS 4 | 管理控制台前端，静态导出模式 |

---

## 功能特性

### 服务器管理
- 多服务器管理（CRUD）
- RCON 远程命令控制台（连接池、优先级队列、自动重连）
- 实时玩家列表 / 小队 / 队伍状态（WebSocket 推送）
- 服务器健康监控（在线 / 降级 / 离线、24h 统计）
- 服务器规则配置（Admins.cfg / Bans.cfg 自动生成）

### 玩家管理
- 玩家踢出 / 警告 / 封禁 / 解封
- 移除玩家小队、强制切换队伍
- 玩家档案（击杀/死亡/TK统计、武器偏好、战力分析）
- 跨服务器玩家搜索
- 身份解析与小号检测（Union-Find 并查集算法）

### 自动化规则
- **工作流引擎**：可配置的触发器 → 条件判断 → 动作执行
- **播种模式**：低人数时自动限制武器/载具/工事
- **队伍平衡**：自动/手动阵营洗牌
- **AFK 管理**：自动检测并处理挂机玩家
- **聊天审核**：脏话过滤、递进式处罚
- **定时广播**：MOTD、多条公告轮播
- **自动回复**：关键词触发机器人回复
- **玩家进入提醒**：自定义欢迎消息

### 事件日志
- 击杀 / 死亡 / TK 事件记录
- 聊天消息记录（含频道分类）
- 飞行 / 爆炸 / 部署物 / 载具事件
- 比赛事件（开局/结束、队伍票数）
- 服务器日志（等级筛选）
- RCON 命令操作日志

### 伤害与通知
- 友军伤害通知（TK / 误伤 / 击杀）
- 自定义消息模板（支持 `{{attacker}}` `{{victim}}` `{{damage}}` `{{weapon}}` 等变量）
- TK 道歉机制（倒计时 + 关键词检测）
- 异常伤害检测与告警

### 权限系统
- 多级权限（超级管理员 / 管理员 / 观察者 / 自定义组）
- 权限模板复制
- 游戏内 Admin 角色自动同步
- JWT 认证 + 速率限制

### 审计与运维
- 配置变更历史（审计日志、版本回滚）
- 操作日志（谁在何时做了什么）
- 系统日志（后端/Agent 运行状态）

---

## 目录结构

```
newuisquad/
├── backend/                    # Rust 后端服务
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs             # 服务入口 & 启动流程
│   │   ├── lib.rs              # 模块导出
│   │   ├── config.rs           # 环境变量配置
│   │   ├── db.rs               # PostgreSQL 连接池
│   │   ├── api/                # API 路由处理（34 个模块）
│   │   │   ├── mod.rs          # 路由定义 & AppState
│   │   │   ├── auth.rs         # 认证
│   │   │   ├── servers.rs      # 服务器 CRUD
│   │   │   ├── bans.rs         # 封禁管理
│   │   │   ├── permissions.rs  # 权限管理
│   │   │   ├── player_tracker.rs  # 实时追踪
│   │   │   ├── workflows.rs   # 工作流
│   │   │   └── ... (34 个模块)
│   │   ├── models/             # 数据模型（18 个）
│   │   ├── services/           # 业务逻辑（30 个服务）
│   │   │   ├── broadcast_handler.rs  # 广播处理
│   │   │   ├── damage_notify_service.rs  # 伤害通知
│   │   │   ├── player_tracker.rs    # 玩家追踪
│   │   │   ├── chat_automod.rs      # 聊天审核
│   │   │   ├── event_manager.rs     # 事件管理
│   │   │   ├── identity_resolver.rs # 身份解析
│   │   │   ├── workflow_engine.rs   # 工作流引擎
│   │   │   ├── seeding_service.rs   # 播种模式
│   │   │   ├── team_balance_service.rs  # 队伍平衡
│   │   │   ├── afk_service.rs       # AFK 管理
│   │   │   ├── ban_enforcer.rs      # Ban 强制执行
│   │   │   ├── log_batcher.rs       # 批量日志写入
│   │   │   └── ... (30 个服务)
│   │   ├── repositories/      # 数据访问层（13 个）
│   │   ├── rcon_client/       # RCON 连接池
│   │   └── log_watcher/       # 日志监控
│   ├── migrations/            # 41 个 SQL 迁移文件
│   └── .env                   # 环境变量配置
│
├── agent/                     # 游戏服务器 Agent
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs            # Agent 入口
│       ├── config.rs          # Agent 配置
│       ├── ws_client.rs       # WebSocket 客户端
│       ├── rcon_listener.rs   # RCON 长连接监听
│       ├── log_watcher.rs     # 日志文件监控
│       ├── file_ops.rs        # 文件操作
│       └── protocol.rs        # 通信协议
│
├── admin-console/             # 管理控制台前端
│   ├── package.json
│   ├── next.config.ts         # 静态导出配置
│   ├── app/
│   │   ├── layout.tsx         # 根布局（暗色主题）
│   │   ├── page.tsx           # 主页面（动态路由）
│   │   ├── globals.css        # 全局样式
│   │   ├── types.ts           # TypeScript 类型定义
│   │   ├── lib/
│   │   │   └── api.ts         # API 请求封装
│   │   └── components/
│   │       ├── Sidebar.tsx    # 侧边导航
│   │       ├── Topbar.tsx     # 顶部栏
│   │       ├── LoginPage.tsx  # 登录页
│   │       └── pages/        # 22 个页面组件
│   │           ├── ControlPanelPage.tsx  # 控制面板
│   │           ├── ConfigPanelPage.tsx   # 配置面板
│   │           ├── ChatLogsPage.tsx      # 聊天日志
│   │           ├── KillLogsPage.tsx      # 击杀日志
│   │           ├── BanManagementPage.tsx # 封禁管理
│   │           ├── RconConsolePage.tsx   # RCON 控制台
│   │           ├── WorkflowsPage.tsx     # 工作流
│   │           └── ... (22 个页面)
│   └── out/                   # 构建输出目录
│
├── docs/                      # 设计文档
└── web/                       # 静态部署产物
```

---

## 环境要求

| 依赖 | 版本 |
|------|------|
| Rust | 1.75+ (edition 2021) |
| PostgreSQL | 14+ |
| Node.js | 20+ |
| npm | 10+ |

---

## 快速开始

### 1. 克隆项目并安装依赖

```bash
git clone <repository-url>
cd newuisquad
```

### 2. 配置数据库

创建 PostgreSQL 数据库并配置环境变量：

```bash
# 创建数据库
createdb newsquad

# 配置环境变量
cp backend/.env backend/.env  # 已存在则直接编辑
```

编辑 `backend/.env`：

```env
DATABASE_URL=postgres://用户名:密码@数据库地址:5432/数据库名
SERVER_HOST=0.0.0.0
SERVER_PORT=8000
STEAM_API_KEY=你的Steam_API_Key
INIT_ADMIN_USERNAME=admin
INIT_ADMIN_PASSWORD=你的安全密码
JWT_SECRET=你的JWT密钥
ALLOWED_ORIGIN=*
```

### 3. 启动后端

```bash
cd backend
cargo build --release
cargo run --release
```

首次启动会自动：
- 执行数据库迁移（创建所有表）
- 创建默认管理员账号

后端默认监听 `http://0.0.0.0:8000`。

### 4. 构建前端（可选）

```bash
cd admin-console
npm install
npm run build    # 生成静态文件到 out/ 目录
```

前端构建为纯静态 HTML，通过 Nginx 或后端反向代理提供服务。

### 5. 部署 Agent（在游戏服务器上）

```bash
cd agent
cargo build --release
```

将编译好的 `game-server-agent` 部署到 Squad 服务器，配置 WebSocket 连接地址指向后端。

---

## API 概览

所有受保护的 API 需要在请求头中携带 Bearer Token：

```
Authorization: Bearer <jwt_token>
```

### 认证

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/auth/login` | 登录获取 Token |
| POST | `/api/v1/auth/verify` | 验证 Token |

### 服务器管理

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/servers` | 服务器列表 |
| POST | `/api/v1/servers` | 添加服务器 |
| GET | `/api/v1/servers/{id}` | 服务器详情 |
| PUT | `/api/v1/servers/{id}` | 更新服务器 |
| DELETE | `/api/v1/servers/{id}` | 删除服务器 |
| GET | `/api/v1/servers/{id}/health` | 服务器健康状态 |
| GET | `/api/v1/servers/{id}/stats` | 24h 统计数据 |
| GET | `/api/v1/servers/enhanced` | 增强版列表（含健康状态） |
| GET | `/api/v1/servers-health` | 全部服务器健康汇总 |

### RCON

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/servers/{id}/rcon` | 执行 RCON 命令 |
| GET | `/api/v1/servers/{id}/rcon-logs` | RCON 操作日志 |

### 玩家追踪

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/servers/{id}/live-state` | 实时服务器状态 |
| GET | `/api/v1/servers/{id}/live-players` | 在线玩家列表 |
| GET | `/api/v1/servers/{id}/live-squads` | 小队列表 |
| GET | `/api/v1/servers/{id}/live-teams` | 队伍列表 |
| POST | `/api/v1/servers/{id}/live-refresh` | 手动刷新 |
| GET | `/api/v1/players/search` | 跨服务器搜索玩家 |

### 玩家操作

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/servers/{id}/player-action` | 玩家操作（踢出/警告/封禁/切换队伍等） |
| GET | `/api/v1/servers/{id}/bans` | 封禁列表 |
| GET | `/api/v1/servers/{id}/warns` | 警告列表 |
| POST | `/api/v1/servers/{id}/ban-player` | 封禁玩家 |
| GET | `/api/v1/servers/{id}/ban-list` | 全面封禁列表（RCON+DB+文件） |
| DELETE | `/api/v1/servers/{id}/disband-squad/{team_id}/{squad_id}` | 解散小队 |

### 事件日志

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/servers/{id}/kill-events` | 击杀事件 |
| GET | `/api/v1/servers/{id}/fly-events` | 飞行事件 |
| GET | `/api/v1/servers/{id}/match-events` | 比赛事件 |
| GET | `/api/v1/servers/{id}/match-summaries` | 比赛摘要 |
| GET | `/api/v1/servers/{id}/explosion-events` | 爆炸事件 |
| GET | `/api/v1/servers/{id}/deployable-events` | 部署物事件 |
| GET | `/api/v1/servers/{id}/vehicle-events` | 载具事件 |
| GET | `/api/v1/servers/{id}/chat-messages` | 聊天消息 |
| GET | `/api/v1/servers/{id}/logs` | 服务器日志 |
| GET | `/api/v1/servers/{id}/logs/stream` | 日志 WebSocket 流 |

### 配置管理

| 方法 | 路径 | 说明 |
|------|------|------|
| GET/PUT | `/api/v1/servers/{id}/tk-settings` | TK 设置 |
| GET/PUT | `/api/v1/servers/{id}/afk-settings` | AFK 设置 |
| GET/PUT | `/api/v1/servers/{id}/broadcast-settings` | 广播设置 |
| GET/PUT | `/api/v1/servers/{id}/team-settings` | 队伍设置 |
| GET/PUT | `/api/v1/servers/{id}/seed-settings` | 播种设置 |
| GET/PUT | `/api/v1/servers/{id}/damage-notify-settings` | 伤害通知设置 |
| GET/PUT | `/api/v1/servers/{id}/abnormal-damage-config` | 异常伤害配置 |
| GET/PUT | `/api/v1/servers/{id}/team-switch-config` | 队伍切换配置 |
| GET/PUT | `/api/v1/servers/{id}/chat-moderation-settings` | 聊天审核设置 |

### 权限

| 方法 | 路径 | 说明 |
|------|------|------|
| GET/POST | `/api/v1/servers/{id}/permission-groups` | 权限组列表/创建 |
| PUT/DELETE | `/api/v1/servers/{id}/permission-groups/{gid}` | 更新/删除权限组 |
| GET/POST | `/api/v1/servers/{id}/permission-admins` | 管理员列表/添加 |
| PUT/DELETE | `/api/v1/servers/{id}/permission-admins/{aid}` | 更新/删除管理员 |
| GET | `/api/v1/servers/{id}/Admins.cfg` | 导出 Admins.cfg |
| GET | `/api/v1/servers/{id}/Bans.cfg` | 导出 Bans.cfg |

### 工作流

| 方法 | 路径 | 说明 |
|------|------|------|
| GET/POST | `/api/v1/servers/{id}/workflows` | 工作流列表/创建 |
| GET/PUT/DELETE | `/api/v1/servers/{id}/workflows/{wid}` | 获取/更新/删除工作流 |
| POST | `/api/v1/servers/{id}/workflows/{wid}/toggle` | 启用/禁用 |
| GET | `/api/v1/servers/{id}/workflows/{wid}/executions` | 执行历史 |

### 其他

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/identity/compute` | 计算身份关联 |
| GET | `/api/v1/identity/lookup` | 身份查询 |
| GET | `/api/v1/identities` | 身份列表 |
| GET | `/api/v1/player-profile/{steam64}` | 玩家档案 |
| GET | `/api/v1/operation-logs` | 操作日志 |
| GET | `/api/v1/audit-stats` | 审计统计 |
| GET | `/api/v1/audit-detail` | 审计详情 |
| GET | `/api/v1/servers/{id}/config-history` | 配置变更历史 |

---

## Agent 部署

Agent 部署在 Squad 游戏服务器上，负责：

1. **日志监控**：实时读取 Squad 服务器日志文件，解析事件并上报后端
2. **RCON 监听**：与后端保持 WebSocket 长连接，接收并执行 RCON 命令
3. **文件操作**：远程读取/写入服务器配置文件（Admins.cfg、Bans.cfg 等）

### Agent 环境变量

```env
BACKEND_WS_URL=wss://你的后端地址/agent/connect
AGENT_TOKEN=你的Agent认证Token
LOG_FILE_PATH=/path/to/SquadGame/Saved/Logs/SquadGame.log
RCON_HOST=127.0.0.1
RCON_PORT=21114
RCON_PASSWORD=你的RCON密码
```

---

## 数据库

项目使用 PostgreSQL，包含 41 个迁移文件，涵盖以下核心表：

| 类别 | 表名 |
|------|------|
| **配置** | `servers`, `broadcast_settings`, `tk_settings`, `afk_settings`, `seed_settings`, `team_settings`, `team_switch_config`, `damage_notify_settings`, `abnormal_damage_config`, `chat_moderation_settings` |
| **事件日志** | `kill_events`, `fly_events`, `chat_messages`, `explosion_events`, `deployable_damaged_events`, `tick_rate_events`, `vehicle_events`, `match_info`, `revive_events` |
| **玩家** | `player_info`, `player_identities`, `player_identity_lookup` |
| **权限** | `permission_groups`, `permission_admins`, `admin_users` |
| **自动化** | `announcements`, `auto_replies`, `abnormal_damage_rules`, `workflows`, `workflow_executions` |
| **审计** | `rcon_logs`, `server_logs`, `admin_actions`, `system_logs`, `config_history`, `audit_config` |
| **其他** | `file_ops`, `chat_violations`, `bans`, `squad_creations`, `team_assignments` |

---

## 开发

### 后端

```bash
cd backend
cargo run              # 开发模式运行
cargo build --release  # 生产构建
cargo test             # 运行测试
```

### 前端

```bash
cd admin-console
npm run dev            # 开发服务器（http://localhost:3000）
npm run build          # 静态导出到 out/
npm run lint           # 代码检查
```

### Agent

```bash
cd agent
cargo run              # 开发模式运行
cargo build --release  # 生产构建
```

---

## 部署建议

### 生产环境架构

```
                   ┌──────────┐
                   │  Nginx   │ 反向代理 + 静态文件
                   └────┬─────┘
                        │
          ┌─────────────┼─────────────┐
          │             │             │
    /api/* │      /*.html│     /agent/connect
          ▼             ▼             │
    ┌──────────┐  ┌──────────┐       │
    │ Backend  │  │  静态文件  │       │
    │ :8000    │  │  (out/)   │       │
    └────┬─────┘  └──────────┘       │
         │                            │
         ▼                            ▼
    ┌──────────┐              ┌──────────────┐
    │PostgreSQL│              │ Squad Server  │
    │          │              │   + Agent     │
    └──────────┘              └──────────────┘
```

### Nginx 配置示例

```nginx
server {
    listen 80;
    server_name admin.example.com;

    # 管理控制台静态文件
    root /opt/newuisquad/admin-console/out;
    index index.html;

    # API 反向代理
    location /api/ {
        proxy_pass http://127.0.0.1:8000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    # Agent WebSocket
    location /agent/connect {
        proxy_pass http://127.0.0.1:8000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
    }

    # SPA fallback
    location / {
        try_files $uri /index.html;
    }
}
```

---

## 安全注意事项

1. **JWT_SECRET**：在生产环境中务必修改为随机强密钥
2. **INIT_ADMIN_PASSWORD**：首次启动后立即修改默认管理员密码
3. **STEAM_API_KEY**：保管好 Steam Web API Key，避免泄漏
4. **HTTPS**：生产环境建议启用 TLS/SSL
5. **数据库**：定期备份 PostgreSQL 数据
6. **RCON 密码**：Agent 与后端之间的 RCON 通信建议通过内网
