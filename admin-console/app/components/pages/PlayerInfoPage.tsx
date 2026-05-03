'use client';

import { useState, useEffect } from 'react';

const API_BASE = '/api/v1';

interface PlayerInfo { server_id: number; player_name: string; steam64: string; eos_id: string; ip: string; first_seen: string; last_seen: string; }

export default function PlayerInfoPage() {
  const [servers, setServers] = useState<{ id: number; name: string }[]>([]);
  const [serverId, setServerId] = useState<number | null>(null);
  const [players, setPlayers] = useState<PlayerInfo[]>([]);
  const [page, setPage] = useState(1);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(false);
  const [search, setSearch] = useState('');

  useEffect(() => {
    fetch(`${API_BASE}/servers`).then(r => r.json())
      .then(d => { setServers(d.data || []); if (d.data?.length > 0) setServerId(d.data[0].id); })
      .catch(() => {});
  }, []);

  useEffect(() => {
    if (!serverId) return; setLoading(true);
    const qs = search ? `&steam64=${encodeURIComponent(search)}` : '';
    fetch(`${API_BASE}/servers/${serverId}/player-info?page=${page}&per_page=50${qs}`).then(r => r.json())
      .then(d => { setPlayers(d.data || []); setTotal(d.total || 0); setLoading(false); }).catch(() => setLoading(false));
  }, [serverId, page, search]);

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
        <div className="card-header">
          <div><div className="card-title">玩家信息</div><div className="card-sub">玩家登录记录（共 {total} 名）</div></div>
          <div style={{ display: 'flex', gap: 8 }}>
            <input className="rcon-input" style={{ width: 180, fontSize: 12 }} placeholder="搜索玩家名或 Steam64..."
              value={search} onChange={e => setSearch(e.target.value)}
              onKeyDown={e => e.key === 'Enter' && setPage(1)} />
            <button className="rcon-btn" style={{ width: 'auto', padding: '6px 12px', fontSize: 12 }} onClick={() => { setPage(1); }}>搜索</button>
          </div>
        </div>
        <div className="card-body" style={{ padding: 0 }}>
          {loading ? <div style={{ padding: 40, textAlign: 'center', color: 'var(--text3)' }}>加载中...</div>
          : players.length === 0 ? <div className="empty-state"><h3>暂无玩家记录</h3></div>
          : <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
            <thead><tr style={{ borderBottom: '2px solid var(--border)', textAlign: 'left' }}>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>玩家名称</th>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>Steam64</th>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>EOS ID</th>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>IP 地址</th>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>首次登录</th>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>最近登录</th>
            </tr></thead>
            <tbody>{players.map((p, i) => (
              <tr key={i} style={{ borderBottom: '1px solid var(--border)' }}>
                <td style={{ padding: '8px 14px', fontWeight: 500 }}>{p.player_name || '-'}</td>
                <td style={{ padding: '8px 14px', fontFamily: 'monospace', fontSize: 12, color: 'var(--text2)' }}>{p.steam64 || '-'}</td>
                <td style={{ padding: '8px 14px', fontFamily: 'monospace', fontSize: 11, color: 'var(--text2)' }} title={p.eos_id}>{(p.eos_id || '').slice(0, 16)}...</td>
                <td style={{ padding: '8px 14px', fontFamily: 'monospace', fontSize: 12 }}>{p.ip || '-'}</td>
                <td style={{ padding: '8px 14px', whiteSpace: 'nowrap', fontSize: 12 }}>{new Date(p.first_seen).toLocaleString()}</td>
                <td style={{ padding: '8px 14px', whiteSpace: 'nowrap', fontSize: 12 }}>{new Date(p.last_seen).toLocaleString()}</td>
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
