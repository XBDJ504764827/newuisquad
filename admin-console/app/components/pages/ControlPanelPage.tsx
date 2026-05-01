'use client';

import { useState, useEffect, useRef, useCallback } from 'react';

const API_BASE = 'http://192.168.0.137:8000/api/v1';

interface Server {
  id: number;
  server_id: string;
  name: string;
  ip: string;
  rcon_port: number;
  created_at: string;
  token?: string;
}

interface LogEntry {
  log_level: string;
  category: string | null;
  message: string;
  raw_line: string | null;
  logged_at: string;
}

export default function ControlPanelPage() {
  const [servers, setServers] = useState<Server[]>([]);
  const [selectedServer, setSelectedServer] = useState<Server | null>(null);
  const [showAddModal, setShowAddModal] = useState(false);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [rconCommand, setRconCommand] = useState('');
  const [rconResult, setRconResult] = useState('');
  const [loading, setLoading] = useState(true);
  const wsRef = useRef<WebSocket | null>(null);
  const logsEndRef = useRef<HTMLDivElement>(null);

  const [form, setForm] = useState({ name: '', ip: '', rcon_port: 28016, rcon_password: '', admin_user: 'Admin' });
  const [submitting, setSubmitting] = useState(false);
  const [newToken, setNewToken] = useState('');
  const [error, setError] = useState('');

  useEffect(() => {
    fetch(`${API_BASE}/servers`)
      .then(r => r.json())
      .then(data => {
        setServers(data.data || []);
        if (data.data?.length > 0) setSelectedServer(data.data[0]);
        setLoading(false);
      })
      .catch(() => setLoading(false));
  }, []);

  useEffect(() => {
    if (!selectedServer) return;
    if (wsRef.current) wsRef.current.close();
    const ws = new WebSocket(`ws://192.168.0.137:8000/api/v1/servers/${selectedServer.id}/logs/stream`);
    ws.onmessage = (e) => {
      try {
        const entry: LogEntry = JSON.parse(e.data);
        setLogs(prev => [...prev.slice(-200), entry]);
      } catch {}
    };
    wsRef.current = ws;
    setLogs([]);
    return () => ws.close();
  }, [selectedServer?.id]);

  useEffect(() => { logsEndRef.current?.scrollIntoView({ behavior: 'smooth' }); }, [logs]);

  const sendRcon = useCallback(async () => {
    if (!selectedServer || !rconCommand) return;
    const res = await fetch(`${API_BASE}/servers/${selectedServer.id}/rcon`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ command: rconCommand, admin_user: 'Admin' }),
    });
    const data = await res.json();
    setRconResult(data.response || data.error || 'OK');
    setRconCommand('');
  }, [selectedServer, rconCommand]);

  const handleAddServer = useCallback(async () => {
    setSubmitting(true);
    setError('');
    const res = await fetch(`${API_BASE}/servers`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(form),
    });
    const data = await res.json();
    setSubmitting(false);
    if (data.error) {
      setError(data.error);
    } else {
      setNewToken(data.token);
      setServers(prev => [...prev, data]);
      setSelectedServer(data);
    }
  }, [form]);

  if (loading) return (
    <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      <div className="card"><div className="card-body" style={{ textAlign: 'center', color: 'var(--text3)' }}>加载中...</div></div>
    </div>
  );

  if (servers.length === 0 && !showAddModal) {
    return (
      <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
        <div className="card">
          <div className="empty-state">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
              <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><line x1="12" y1="8" x2="12" y2="16"/><line x1="8" y1="12" x2="16" y2="12"/>
            </svg>
            <h3>暂无服务器</h3>
            <p style={{ marginTop: 8 }}>点击下方按钮添加您的第一台游戏服务器。</p>
            <button className="rcon-btn" style={{ marginTop: 20, width: 'auto', padding: '10px 24px' }} onClick={() => setShowAddModal(true)}>
              添加服务器
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      {servers.length > 0 && (
        <div style={{ display: 'flex', gap: 8, alignItems: 'center', flexWrap: 'wrap' }}>
          {servers.map(s => (
            <button key={s.id}
              className={`tab-btn ${selectedServer?.id === s.id ? 'active' : ''}`}
              style={{ borderBottom: selectedServer?.id === s.id ? '2px solid var(--text)' : '2px solid transparent' }}
              onClick={() => setSelectedServer(s)}>
              {s.name}
            </button>
          ))}
          <button className="icon-btn" style={{ marginLeft: 4 }} title="添加服务器" onClick={() => { setShowAddModal(true); setNewToken(''); setError(''); }}>
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
          </button>
        </div>
      )}

      <div className="control-panel-layout" style={{ display: 'grid', gridTemplateColumns: '350px 1fr', gap: 20, alignItems: 'start' }}>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
          {selectedServer && (
            <div className="card">
              <div className="card-header"><div><div className="card-title">服务器信息</div></div></div>
              <div className="card-body" style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', borderBottom: '1px solid var(--border)', paddingBottom: 12 }}>
                  <span style={{ color: 'var(--text3)', fontSize: 12 }}>服务器 ID</span>
                  <span style={{ fontWeight: 600 }}>{selectedServer.server_id}</span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', borderBottom: '1px solid var(--border)', paddingBottom: 12 }}>
                  <span style={{ color: 'var(--text3)', fontSize: 12 }}>服务器名称</span>
                  <span style={{ fontWeight: 600 }}>{selectedServer.name}</span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', borderBottom: '1px solid var(--border)', paddingBottom: 12 }}>
                  <span style={{ color: 'var(--text3)', fontSize: 12 }}>服务器 IP</span>
                  <span style={{ fontWeight: 600 }}>{selectedServer.ip}</span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <span style={{ color: 'var(--text3)', fontSize: 12 }}>RCON 端口</span>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                    <span style={{ fontWeight: 600 }}>{selectedServer.rcon_port}</span>
                    <span className="badge green">已连接</span>
                  </div>
                </div>
              </div>
            </div>
          )}

          {selectedServer && (
            <div className="card">
              <div className="card-header"><div><div className="card-title">RCON 远程控制</div><div className="card-sub">向服务器发送 RCON 命令</div></div></div>
              <div className="card-body">
                <div className="rcon-input-group">
                  <input type="text" className="rcon-input" placeholder="输入指令 (例: ListPlayers)..." value={rconCommand}
                    onChange={e => setRconCommand(e.target.value)} onKeyDown={e => e.key === 'Enter' && sendRcon()} />
                  <button className="rcon-btn" onClick={sendRcon}>发送命令</button>
                  {rconResult && <div className="terminal" style={{ marginTop: 12, maxHeight: 120, overflowY: 'auto', fontSize: 12, padding: 10 }}>{rconResult}</div>}
                </div>
              </div>
            </div>
          )}
        </div>

        <div className="card" style={{ height: '100%', minHeight: 500 }}>
          <div className="card-header">
            <div><div className="card-title">服务器实时日志</div><div className="card-sub">{selectedServer ? selectedServer.name : ''} - 实时同步</div></div>
            <div style={{ display: 'flex', gap: 8 }}>
              <span className="badge gray" style={{ cursor: 'pointer' }} onClick={() => setLogs([])}>清空</span>
            </div>
          </div>
          <div className="card-body" style={{ padding: 0, display: 'flex', flexDirection: 'column' }}>
            <div className="terminal" style={{ flex: 1, border: 'none', borderRadius: '0 0 8px 8px', minHeight: 450 }}>
              {logs.length === 0 && <div style={{ color: 'var(--text3)' }}>等待日志数据...</div>}
              {logs.map((entry, i) => (
                <div key={i}>
                  <span className="time">[{new Date(entry.logged_at).toLocaleTimeString()}]</span>
                  <span className={entry.log_level === 'ERROR' ? 'error' : entry.log_level === 'WARN' ? 'warn' : entry.log_level === 'SUCCESS' ? 'success' : 'info'}>
                    [{entry.category || 'General'}]
                  </span> {entry.message}
                </div>
              ))}
              <div ref={logsEndRef} style={{ animation: 'pulse 1.5s infinite', color: 'var(--text3)', marginTop: 4 }}>_</div>
            </div>
          </div>
        </div>
      </div>

      {showAddModal && (
        <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.6)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 1000 }}
          onClick={e => { if (e.target === e.currentTarget) setShowAddModal(false); }}>
          <div style={{ background: 'var(--bg2)', border: '1px solid var(--border)', borderRadius: 'var(--radius)', padding: 24, width: 420, maxWidth: '90vw' }}
            onClick={e => e.stopPropagation()}>
            <h3 style={{ marginBottom: 20, fontSize: 16 }}>添加游戏服务器</h3>

            {newToken ? (
              <div>
                <div className="terminal" style={{ padding: 16, marginBottom: 16 }}>
                  <span className="success">服务器添加成功！</span><br/><br/>
                  Agent Token（复制到 agent .env 文件）：<br/>
                  <code style={{ color: '#22c55e', wordBreak: 'break-all', userSelect: 'all' }}>{newToken}</code>
                </div>
                <button className="rcon-btn" onClick={() => setShowAddModal(false)}>关闭</button>
              </div>
            ) : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                <label style={{ fontSize: 12, color: 'var(--text2)' }}>服务器名称</label>
                <input className="rcon-input" value={form.name} onChange={e => setForm({...form, name: e.target.value})} placeholder="华东区-狂欢生存服" />
                <label style={{ fontSize: 12, color: 'var(--text2)' }}>服务器 IP</label>
                <input className="rcon-input" value={form.ip} onChange={e => setForm({...form, ip: e.target.value})} placeholder="121.40.123.45" />
                <label style={{ fontSize: 12, color: 'var(--text2)' }}>RCON 端口</label>
                <input className="rcon-input" type="number" value={form.rcon_port} onChange={e => setForm({...form, rcon_port: parseInt(e.target.value) || 28016})} />
                <label style={{ fontSize: 12, color: 'var(--text2)' }}>RCON 密码</label>
                <input className="rcon-input" type="password" value={form.rcon_password} onChange={e => setForm({...form, rcon_password: e.target.value})} placeholder="••••••" />
                {error && <div style={{ color: 'var(--red)', fontSize: 12 }}>{error}</div>}
                <div style={{ display: 'flex', gap: 8, marginTop: 8 }}>
                  <button className="rcon-btn" onClick={handleAddServer} disabled={submitting} style={{ opacity: submitting ? 0.5 : 1 }}>
                    {submitting ? '验证中...' : '验证并添加'}
                  </button>
                  <button className="rcon-btn" style={{ background: 'var(--bg3)', color: 'var(--text2)' }} onClick={() => setShowAddModal(false)}>取消</button>
                </div>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
