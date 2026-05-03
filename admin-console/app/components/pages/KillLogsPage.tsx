'use client';

import { useState, useEffect } from 'react';

const API_BASE = '/api/v1';

interface KillEvent { id: number; attacker_name: string; attacker_eos: string; attacker_steam64: string; victim_name: string; damage: number; weapon: string; is_kill: boolean; is_teamkill: boolean; logged_at: string; }

export default function KillLogsPage() {
  const [servers, setServers] = useState<{ id: number; name: string }[]>([]);
  const [serverId, setServerId] = useState<number | null>(null);
  const [events, setEvents] = useState<KillEvent[]>([]);
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
    fetch(`${API_BASE}/servers/${serverId}/kill-events?page=${page}&per_page=50`).then(r => r.json())
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
        <div className="card-header"><div><div className="card-title">击倒记录</div><div className="card-sub">战斗伤害日志（共 {total} 条）</div></div></div>
        <div className="card-body" style={{ padding: 0 }}>
          {loading ? <div style={{ padding: 40, textAlign: 'center', color: 'var(--text3)' }}>加载中...</div>
          : events.length === 0 ? <div className="empty-state"><h3>暂无记录</h3></div>
          : <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
            <thead><tr style={{ borderBottom: '2px solid var(--border)', textAlign: 'left' }}>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>时间</th>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>攻击者</th>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>攻击者 Steam64</th>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>武器</th>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>伤害值</th>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>受害者</th>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>类型</th>
            </tr></thead>
            <tbody>{events.map(e => (
              <tr key={e.id} style={{ borderBottom: '1px solid var(--border)' }}>
                <td style={{ padding: '8px 14px', whiteSpace: 'nowrap', fontSize: 12 }}>{new Date(e.logged_at).toLocaleString()}</td>
                <td style={{ padding: '8px 14px', fontWeight: 500 }}>{e.attacker_name || '-'}</td>
                <td style={{ padding: '8px 14px', fontFamily: 'monospace', fontSize: 12, color: 'var(--text2)' }}>{e.attacker_steam64 || '-'}</td>
                <td style={{ padding: '8px 14px' }}><span className="badge blue" style={{ fontSize: 10 }}>{e.weapon}</span></td>
                <td style={{ padding: '8px 14px', color: e.is_teamkill ? 'var(--red)' : '#f59e0b', fontWeight: 600 }}>{e.damage.toFixed(1)}</td>
                <td style={{ padding: '8px 14px', fontWeight: 500 }}>{e.victim_name}</td>
                <td style={{ padding: '8px 14px' }}>
                  {e.is_teamkill ? <span className="badge red" style={{ fontSize: 10 }}>TK</span>
                   : e.is_kill ? <span className="badge green" style={{ fontSize: 10 }}>击倒</span>
                   : <span className="badge gray" style={{ fontSize: 10 }}>伤害</span>}
                </td>
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
