'use client';

export default function ControlPanelPage() {
  return (
    <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      <div className="control-panel-layout" style={{ display: 'grid', gridTemplateColumns: '350px 1fr', gap: 20, alignItems: 'start' }}>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
          <div className="card">
            <div className="card-header">
              <div><div className="card-title">服务器信息</div></div>
            </div>
            <div className="card-body" style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', borderBottom: '1px solid var(--border)', paddingBottom: 12 }}>
                <span style={{ color: 'var(--text3)', fontSize: 12 }}>服务器 ID</span>
                <span style={{ fontWeight: 600 }}>SRV-CN-001</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', borderBottom: '1px solid var(--border)', paddingBottom: 12 }}>
                <span style={{ color: 'var(--text3)', fontSize: 12 }}>服务器名称</span>
                <span style={{ fontWeight: 600 }}>华东区-狂欢生存服</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', borderBottom: '1px solid var(--border)', paddingBottom: 12 }}>
                <span style={{ color: 'var(--text3)', fontSize: 12 }}>服务器 IP</span>
                <span style={{ fontWeight: 600 }}>121.40.123.45</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <span style={{ color: 'var(--text3)', fontSize: 12 }}>RCON 端口/状态</span>
                <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                  <span style={{ fontWeight: 600 }}>28016</span>
                  <span className="badge green">已连接</span>
                </div>
              </div>
            </div>
          </div>

          <div className="card">
            <div className="card-header">
              <div><div className="card-title">RCON 远程控制</div><div className="card-sub">向服务器发送 RCON 命令</div></div>
            </div>
            <div className="card-body">
              <div className="rcon-input-group">
                <input type="text" className="rcon-input" placeholder="输入指令 (例: status, kick)..." />
                <button className="rcon-btn">发送命令</button>
              </div>
            </div>
          </div>
        </div>

        <div className="card" style={{ height: '100%', minHeight: 500 }}>
          <div className="card-header">
            <div><div className="card-title">服务器实时日志</div><div className="card-sub">游戏端控制台数据实时同步</div></div>
            <div style={{ display: 'flex', gap: 8 }}>
              <span className="badge blue" style={{ cursor: 'pointer' }}>自动滚动</span>
              <span className="badge gray" style={{ cursor: 'pointer' }}>清空</span>
            </div>
          </div>
          <div className="card-body" style={{ padding: 0, display: 'flex', flexDirection: 'column' }}>
            <div className="terminal" style={{ flex: 1, border: 'none', borderRadius: '0 0 8px 8px', minHeight: 450 }}>
              <div><span className="time">[10:45:01]</span> <span className="info">[RCON]</span> 成功连接至服务器 121.40.123.45:28016</div>
              <div><span className="time">[10:45:03]</span> <span className="success">[Server]</span> Map loaded successfully.</div>
              <div><span className="time">[10:45:10]</span> <span className="info">[Player]</span> &apos;李大牛&apos; has joined the game.</div>
              <div><span className="time">[10:46:22]</span> <span className="warn">[Log]</span> Player &apos;王大神&apos; is experiencing high ping (150ms).</div>
              <div><span className="time">[10:48:05]</span> <span className="info">[Chat]</span> 李大牛: 大家好啊！</div>
              <div><span className="time">[10:50:11]</span> <span className="error">[Anti-Cheat]</span> Suspicious movement detected from SteamID:76561198000000001</div>
              <div><span className="time">[10:51:00]</span> <span className="info">[Server]</span> Saved game state.</div>
              <div><span className="time">[10:55:33]</span> <span className="info">[Player]</span> &apos;张无忌&apos; triggered objective A.</div>
              <div><span className="time">[10:58:12]</span> <span className="warn">[Log]</span> Warning: Server tickrate dropped below 60.</div>
              <div><span className="time">[11:00:00]</span> <span className="success">[Broadcast]</span> Automatic message sent to all players.</div>
              <div style={{ animation: 'pulse 1.5s infinite', color: 'var(--text3)', marginTop: 4 }}>_</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
