'use client';

import { useState, useEffect } from 'react';

const API_BASE = 'http://192.168.0.137:8000/api/v1';

interface FlyEvent { id: number; player_name: string; eos_id: string; steam64: string; event_type: string; logged_at: string; }

const TYPE_LABELS: Record<string, string> = {
  possess: '进入镜头', unpossess: '退出镜头', admin_camera: '管理员镜头', spectate: '观战',
};

export default function FlyLogsPage() {
  const [servers, setServers] = useState<{ id: number; name: string }[]>([]);
  const [serverId, setServerId] = useState<number | null>(null);
  const [events, setEvents] = useState<FlyEvent[]>([]);
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
    fetch(`${API_BASE}/servers/${serverId}/fly-events?page=${page}&per_page=50`).then(r => r.json())
      .then(d => { setEvents(d.data || []); setTotal(d.total || 0); setLoading(false); }).catch(() => setLoading(false));
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
        <div className="card-header"><div><div className="card-title">飞天记录</div><div className="card-sub">管理员镜头操作记录（共 {total} 条）</div></div></div>
        <div className="card-body" style={{ padding: 0 }}>
          {loading ? <div style={{ padding: 40, textAlign: 'center', color: 'var(--text3)' }}>加载中...</div>
          : events.length === 0 ? <div className="empty-state"><h3>暂无记录</h3></div>
          : <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
            <thead><tr style={{ borderBottom: '2px solid var(--border)', textAlign: 'left' }}>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>时间</th>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>管理员</th>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>Steam64</th>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>EOS ID</th>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>操作类型</th>
            </tr></thead>
            <tbody>{events.map(e => (
              <tr key={e.id} style={{ borderBottom: '1px solid var(--border)' }}>
                <td style={{ padding: '8px 14px', whiteSpace: 'nowrap', fontSize: 12 }}>{new Date(e.logged_at + 'Z').toLocaleString()}</td>
                <td style={{ padding: '8px 14px', fontWeight: 500 }}>{e.player_name || '-'}</td>
                <td style={{ padding: '8px 14px', fontFamily: 'monospace', fontSize: 12, color: 'var(--text2)' }}>{e.steam64 || '-'}</td>
                <td style={{ padding: '8px 14px', fontFamily: 'monospace', fontSize: 11, color: 'var(--text2)' }}>{e.eos_id || '-'}</td>
                <td style={{ padding: '8px 14px' }}><span className={`badge ${e.event_type === 'possess' || e.event_type === 'admin_camera' ? 'green' : 'gray'}`} style={{ fontSize: 11 }}>{TYPE_LABELS[e.event_type] || e.event_type}</span></td>
              </tr>
            ))}</tbody>
          </table>}
          {total > 50 && (
            <div style={{ display: 'flex', justifyContent: 'center', gap: 8, padding: 16 }}>
              <button className="rcon-btn" style={{ width: 'auto', padding: '6px 14px', fontSize: 12 }} disabled={page <= 1} onClick={() => setPage(p => p - 1)}>上一页</button>
              <span style={{ fontSize: 12, color: 'var(--text2)', alignSelf: 'center' }}>第 {page} / {Math.ceil(total / 50)} 页</span>
              <button className="rcon-btn" style={{ width: 'auto', padding: '6px 14px', fontSize: 12 }} disabled={page >= Math.ceil(total / 50)} onClick={() => setPage(p => p + 1)}>下一页</button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
