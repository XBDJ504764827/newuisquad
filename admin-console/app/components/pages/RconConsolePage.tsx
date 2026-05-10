'use client';

import { useState, useEffect, useCallback, useRef } from 'react';
import { api } from '../../lib/api';

interface Server { id: number; server_id: string; name: string; ip: string; rcon_port: number; }
interface CommandInfo {
  name: string; category: string; syntax: string;
  description: string; command_type: 'Admin' | 'Public';
}
interface HistoryEntry { command: string; response: string; timestamp: Date; }

export default function RconConsolePage() {
  const [servers, setServers] = useState<Server[]>([]);
  const [serverId, setServerId] = useState<number | null>(null);
  const [command, setCommand] = useState('');
  const [output, setOutput] = useState('');
  const [loading, setLoading] = useState(false);
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [showHistory, setShowHistory] = useState(false);
  const [showCommands, setShowCommands] = useState(false);
  const [commands, setCommands] = useState<CommandInfo[]>([]);
  const [cmdFilter, setCmdFilter] = useState('all');
  const [cmdSearch, setCmdSearch] = useState('');
  const [selectedCmd, setSelectedCmd] = useState<CommandInfo | null>(null);
  const historyEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    api('/servers').then(r => r.json()).then(d => {
      const list = d.data || [];
      setServers(list);
      if (list.length > 0) setServerId(list[0].id);
    }).catch(() => {});
  }, []);

  const fetchCommands = useCallback(async () => {
    try {
      const res = await api('/command-catalog');
      const data = await res.json();
      const cmds: CommandInfo[] = (data.data || []).map((c: any) => ({
        name: c.Name || c.name || '',
        category: c.Category || c.category || '',
        syntax: c.Syntax || c.syntax || '',
        description: c.Description || c.description || '',
        command_type: (c.CommandType === 1 || c.command_type === 'Admin') ? 'Admin' : 'Public',
      }));
      setCommands(cmds);
    } catch {}
  }, []);

  useEffect(() => { if (showCommands && commands.length === 0) fetchCommands(); }, [showCommands, commands.length, fetchCommands]);

  const execute = async () => {
    if (!serverId || !command.trim()) return;
    setLoading(true);
    try {
      const res = await api(`/servers/${serverId}/rcon`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ command: command.trim() }),
      });
      const data = await res.json();
      let text = '命令已执行（无返回）';
      if (data.error) text = `错误：${data.error}`;
      else if (data.response) text = data.response;
      else if (data.data?.response) text = data.data.response;
      setOutput(text);
      setHistory(prev => [{ command: command.trim(), response: text, timestamp: new Date() }, ...prev.slice(0, 99)]);
    } catch (e: any) {
      setOutput(`请求失败：${e.message}`);
    }
    setLoading(false);
  };

  const filteredCmds = commands.filter(c => {
    if (cmdFilter === 'admin') return c.command_type === 'Admin';
    if (cmdFilter === 'public') return c.command_type === 'Public';
    return true;
  }).filter(c => {
    if (!cmdSearch) return true;
    const q = cmdSearch.toLowerCase();
    return c.name.toLowerCase().includes(q) || c.description.toLowerCase().includes(q) || c.category.toLowerCase().includes(q);
  });

  const selectCmd = (c: CommandInfo) => { setCommand(c.name); setSelectedCmd(c); setShowCommands(false); };

  const styles = {
    container: { padding: 20 },
    header: { display: 'flex', alignItems: 'center', gap: 12, marginBottom: 20, flexWrap: 'wrap' as const },
    select: { padding: '8px 12px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--bg2)', color: 'var(--text)', fontSize: 14, minWidth: 200 },
    btn: { padding: '8px 16px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--text)', color: 'var(--bg)', cursor: 'pointer', fontWeight: 500, fontSize: 13 },
    btnOutline: { padding: '8px 16px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--bg2)', color: 'var(--text)', cursor: 'pointer', fontSize: 13 },
    input: { flex: 1, padding: '10px 14px', borderRadius: 6, border: '1px solid var(--border)', background: '#1e1e1e', color: '#d4d4d4', fontFamily: 'monospace', fontSize: 14, minWidth: 250 },
    terminal: { background: '#1e1e1e', color: '#d4d4d4', padding: 16, borderRadius: 8, border: '1px solid #333', minHeight: 200, maxHeight: 400, overflowY: 'auto' as const, fontFamily: 'monospace', fontSize: 13, whiteSpace: 'pre-wrap' as const, wordBreak: 'break-all' as const },
    cmdInfo: { background: 'var(--bg2)', borderRadius: 8, padding: 12, marginBottom: 16, border: '1px solid var(--border)' },
    modalOverlay: { position: 'fixed' as const, inset: 0, background: 'rgba(0,0,0,0.6)', zIndex: 1000, display: 'flex', alignItems: 'center', justifyContent: 'center' },
    modal: { background: 'var(--bg)', borderRadius: 12, border: '1px solid var(--border)', width: '90vw', maxWidth: 800, maxHeight: '80vh', display: 'flex', flexDirection: 'column' as const, overflow: 'hidden' },
    modalHeader: { display: 'flex', justifyContent: 'space-between', alignItems: 'center', padding: '16px 20px', borderBottom: '1px solid var(--border)' },
    table: { width: '100%', borderCollapse: 'collapse' as const },
    th: { textAlign: 'left' as const, padding: '10px 12px', borderBottom: '1px solid var(--border)', fontSize: 13, color: 'var(--text2)', position: 'sticky' as const, top: 0, background: 'var(--bg)' },
    td: { padding: '10px 12px', borderBottom: '1px solid var(--border)', fontSize: 13 },
    cmdRow: (t: string) => ({ cursor: 'pointer', borderLeft: t === 'Admin' ? '3px solid #ef4444' : '3px solid #22c55e' }),
    badge: (t: string) => ({ display: 'inline-block', padding: '2px 8px', borderRadius: 10, fontSize: 11, fontWeight: 600, background: t === 'Admin' ? 'rgba(239,68,68,0.15)' : 'rgba(34,197,94,0.15)', color: t === 'Admin' ? '#ef4444' : '#22c55e' }),
  };

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h2 style={{ margin: 0, fontSize: 20, fontWeight: 600 }}>RCON 控制台</h2>
        <select value={serverId || ''} onChange={e => setServerId(Number(e.target.value))} style={styles.select}>
          {servers.map(s => <option key={s.id} value={s.id}>{s.name} ({s.ip}:{s.rcon_port})</option>)}
        </select>
      </div>

      <div style={{ display: 'flex', gap: 8, marginBottom: 16, flexWrap: 'wrap' }}>
        <input
          style={styles.input}
          value={command}
          onChange={e => setCommand(e.target.value)}
          onKeyDown={e => { if (e.key === 'Enter' && !loading) execute(); }}
          placeholder="输入 RCON 命令，按 Enter 执行..."
        />
        <button onClick={execute} disabled={loading || !serverId} style={{ ...styles.btn, opacity: loading ? 0.6 : 1 }}>{loading ? '执行中...' : '执行'}</button>
        <button onClick={() => { setShowCommands(true); if (commands.length === 0) fetchCommands(); }} style={styles.btnOutline}>查看全部命令</button>
        <button onClick={() => setShowHistory(true)} disabled={history.length === 0} style={{ ...styles.btnOutline, opacity: history.length === 0 ? 0.4 : 1 }}>历史记录</button>
      </div>

      {selectedCmd && (
        <div style={styles.cmdInfo}>
          <div style={{ fontWeight: 600, fontSize: 14, marginBottom: 4 }}>{selectedCmd.name}</div>
          <div style={{ fontSize: 12, color: 'var(--text2)', marginBottom: 4 }}>语法：{selectedCmd.syntax}</div>
          <div style={{ fontSize: 13, marginBottom: 4 }}>{selectedCmd.description}</div>
          <span style={styles.badge(selectedCmd.command_type)}>{selectedCmd.command_type === 'Admin' ? '管理员命令' : '公共命令'}</span>
        </div>
      )}

      <div style={styles.terminal}>
        {output || <span style={{ color: '#666' }}>等待命令执行...</span>}
      </div>

      {/* All Commands Modal */}
      {showCommands && (
        <div style={styles.modalOverlay} onClick={() => setShowCommands(false)}>
          <div style={styles.modal} onClick={e => e.stopPropagation()}>
            <div style={styles.modalHeader}>
              <h3 style={{ margin: 0, fontSize: 16 }}>全部可用命令</h3>
              <button onClick={() => setShowCommands(false)} style={{ ...styles.btnOutline, fontSize: 14, padding: '4px 10px' }}>关闭</button>
            </div>
            <div style={{ padding: '12px 20px', display: 'flex', gap: 8, flexWrap: 'wrap', borderBottom: '1px solid var(--border)' }}>
              <input style={{ ...styles.input, flex: 1, minWidth: 150 }} placeholder="搜索命令..." value={cmdSearch} onChange={e => setCmdSearch(e.target.value)} />
              {['all', 'admin', 'public'].map(f => (
                <button key={f} onClick={() => setCmdFilter(f)} style={{ ...styles.btnOutline, background: cmdFilter === f ? 'var(--text)' : 'var(--bg2)', color: cmdFilter === f ? 'var(--bg)' : 'var(--text)', padding: '6px 14px', fontSize: 12 }}>
                  {f === 'all' ? '全部' : f === 'admin' ? '管理员' : '公共'}
                </button>
              ))}
            </div>
            <div style={{ overflow: 'auto', flex: 1, padding: '0 20px 20px' }}>
              <table style={styles.table}>
                <thead><tr>
                  <th style={styles.th}>命令</th><th style={styles.th}>分类</th><th style={styles.th}>描述</th><th style={styles.th}>类型</th>
                </tr></thead>
                <tbody>
                  {filteredCmds.map(c => (
                    <tr key={c.name} onClick={() => selectCmd(c)} style={styles.cmdRow(c.command_type)}>
                      <td style={{ ...styles.td, fontFamily: 'monospace', fontWeight: 600 }}>{c.name}</td>
                      <td style={styles.td}>{c.category}</td>
                      <td style={styles.td}>{c.description}</td>
                      <td style={styles.td}><span style={styles.badge(c.command_type)}>{c.command_type === 'Admin' ? '管理员' : '公共'}</span></td>
                    </tr>
                  ))}
                </tbody>
              </table>
              {filteredCmds.length === 0 && <div style={{ textAlign: 'center', padding: 30, color: 'var(--text3)' }}>无匹配命令</div>}
            </div>
          </div>
        </div>
      )}

      {/* History Modal */}
      {showHistory && (
        <div style={styles.modalOverlay} onClick={() => setShowHistory(false)}>
          <div style={styles.modal} onClick={e => e.stopPropagation()}>
            <div style={styles.modalHeader}>
              <h3 style={{ margin: 0, fontSize: 16 }}>命令历史</h3>
              <div style={{ display: 'flex', gap: 8 }}>
                <button onClick={() => setHistory([])} style={{ ...styles.btnOutline, color: '#ef4444', borderColor: '#ef4444', padding: '4px 10px', fontSize: 12 }}>清空历史</button>
                <button onClick={() => setShowHistory(false)} style={{ ...styles.btnOutline, fontSize: 14, padding: '4px 10px' }}>关闭</button>
              </div>
            </div>
            <div style={{ overflow: 'auto', flex: 1, padding: 20 }}>
              {history.length === 0 ? <div style={{ textAlign: 'center', color: 'var(--text3)', padding: 30 }}>暂无命令历史</div> : history.map((e, i) => (
                <div key={i} style={{ marginBottom: 16, borderBottom: '1px solid var(--border)', paddingBottom: 16 }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
                    <code style={{ fontWeight: 600, fontSize: 13 }}>{e.command}</code>
                    <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
                      <span style={{ fontSize: 11, color: 'var(--text3)' }}>{new Date(e.timestamp).toLocaleString()}</span>
                      <button onClick={() => { setCommand(e.command); setShowHistory(false); }} style={{ ...styles.btnOutline, fontSize: 11, padding: '2px 8px' }}>复用</button>
                    </div>
                  </div>
                  <pre style={{ background: '#1e1e1e', color: '#d4d4d4', padding: 10, borderRadius: 6, margin: 0, fontSize: 12, whiteSpace: 'pre-wrap', wordBreak: 'break-all', maxHeight: 120, overflow: 'auto' }}>{e.response}</pre>
                </div>
              ))}
              <div ref={historyEndRef} />
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
