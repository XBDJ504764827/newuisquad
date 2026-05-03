'use client';

import { useState, useEffect } from 'react';

const API_BASE = '/api/v1';

export default function ChatLogsPage() {
  const [servers, setServers] = useState<{ id: number; name: string }[]>([]);
  const [serverId, setServerId] = useState<number | null>(null);
  const [messages, setMessages] = useState<any[]>([]);
  const [page, setPage] = useState(1);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    fetch(`${API_BASE}/servers`).then(r => r.json())
      .then(d => { setServers(d.data || []); if (d.data?.length > 0) setServerId(d.data[0].id); })
      .catch(() => {});
  }, []);

  useEffect(() => {
    if (!serverId) return; setLoading(true);
    fetch(`${API_BASE}/servers/${serverId}/chat-messages?page=${page}&per_page=100`).then(r => r.json())
      .then(d => { setMessages(d.data || []); setTotal(d.total || 0); setLoading(false); }).catch(() => setLoading(false));
  }, [serverId, page]);

  return (
    <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
        <span style={{ fontSize: 12, color: 'var(--text3)' }}>服务器：</span>
        <select className="rcon-input" style={{ width: 'auto', padding: '6px 10px' }} value={serverId || ''}
          onChange={e => { setServerId(parseInt(e.target.value)); setPage(1); }}>
          {servers.map(s => <option key={s.id} value={s.id}>{s.name}</option>)}
        </select>
      </div>
      <div className="card">
        <div className="card-header"><div><div className="card-title">聊天记录</div><div className="card-sub">游戏内玩家交流记录（共 {total} 条）</div></div></div>
        <div className="card-body" style={{ padding: 0 }}>
          {loading ? <div style={{ padding: 40, textAlign: 'center', color: 'var(--text3)' }}>加载中...</div>
          : messages.length === 0 ? <div className="empty-state"><h3>暂无聊天记录</h3></div>
          : <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
            <thead><tr style={{ borderBottom: '2px solid var(--border)', textAlign: 'left' }}>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500, whiteSpace: 'nowrap' }}>时间</th>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>玩家</th>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>频道</th>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>内容</th>
            </tr></thead>
            <tbody>{messages.map((m, i) => (
              <tr key={i} style={{ borderBottom: '1px solid var(--border)' }}>
                <td style={{ padding: '6px 14px', whiteSpace: 'nowrap', fontSize: 12 }}>{new Date(m.logged_at).toLocaleString()}</td>
                <td style={{ padding: '6px 14px', fontWeight: 500 }}>{m.player_name || '-'}</td>
                <td style={{ padding: '6px 14px' }}>
                  <span className={`badge ${m.channel === 'Team' ? 'blue' : 'gray'}`} style={{ fontSize: 10 }}>{m.channel === 'Team' ? '小队' : '全局'}</span>
                </td>
                <td style={{ padding: '6px 14px' }}>{m.message}</td>
              </tr>
            ))}</tbody>
          </table>}
          {total > 100 && (
            <div style={{ display: 'flex', justifyContent: 'center', gap: 8, padding: 16 }}>
              <button className="rcon-btn" style={{ width: 'auto', padding: '6px 14px', fontSize: 12 }} disabled={page <= 1} onClick={() => setPage(p => p - 1)}>上一页</button>
              <span style={{ fontSize: 12, color: 'var(--text2)', alignSelf: 'center' }}>第 {page} / {Math.ceil(total / 100)} 页</span>
              <button className="rcon-btn" style={{ width: 'auto', padding: '6px 14px', fontSize: 12 }} disabled={page >= Math.ceil(total / 100)} onClick={() => setPage(p => p + 1)}>下一页</button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
