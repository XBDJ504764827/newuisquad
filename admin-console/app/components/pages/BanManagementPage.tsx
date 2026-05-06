'use client';

import { useState, useEffect, useCallback } from 'react';

import { useServers } from '../../lib/useServers';
import { api } from '../../lib/api';

interface BanEntry {
  steam_id: string;
  player_name: string;
  duration: string;
  reason: string;
  source: string;
}

export default function BanManagementPage() {
  const { servers } = useServers();
  const [serverId, setServerId] = useState<number | null>(null);
  const [bans, setBans] = useState<BanEntry[]>([]);
  const [loading, setLoading] = useState(false);

  // 封禁表单
  const [steamId, setSteamId] = useState('');
  const [reason, setReason] = useState('');
  const [duration, setDuration] = useState(0); // 0=permanent
  const [lookupName, setLookupName] = useState('');
  const [looking, setLooking] = useState(false);
  const [banning, setBanning] = useState(false);
  const [msg, setMsg] = useState('');

  useEffect(() => {
    if (servers.length > 0 && !serverId) setServerId(servers[0].id);
  }, [servers, serverId]);

  const fetchBans = useCallback(() => {
    if (!serverId) return;
    setLoading(true);
    api(`/servers/${serverId}/ban-list`)
      .then(r => r.json())
      .then(d => { setBans(d.data || []); setLoading(false); })
      .catch(() => setLoading(false));
  }, [serverId]);

  useEffect(() => {
    fetchBans();
  }, [fetchBans]);

  // Steam 名称查询
  const handleLookup = () => {
    if (!steamId || steamId.length < 10) return;
    setLooking(true);
    setLookupName('');
    api(`/steam-player/${steamId}`)
      .then(r => r.json())
      .then(d => {
        if (d.player_name) setLookupName(d.player_name);
        else setLookupName('（未找到）');
        setLooking(false);
      })
      .catch(() => { setLookupName('（查询失败）'); setLooking(false); });
  };

  // 确认封禁
  const handleBan = () => {
    if (!serverId || !steamId || !reason) return;
    setBanning(true);
    api(`/servers/${serverId}/ban-player`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ steam_id: steamId, reason, duration }),
    })
      .then(r => r.json())
      .then(d => {
        setBanning(false);
        if (d.error) { setMsg(d.error); setTimeout(() => setMsg(''), 4000); }
        else {
          setMsg(d.message || '封禁成功');
          setSteamId('');
          setReason('');
          setLookupName('');
          setDuration(0);
          fetchBans();
          setTimeout(() => setMsg(''), 4000);
        }
      })
      .catch(() => { setBanning(false); setMsg('操作失败'); setTimeout(() => setMsg(''), 4000); });
  };

  return (
    <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      {servers.length > 0 && (
        <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
          <span style={{ fontSize: 12, color: 'var(--text3)' }}>服务器：</span>
          <select className="rcon-input" style={{ width: 'auto', padding: '6px 10px' }} value={serverId || ''}
            onChange={e => setServerId(parseInt(e.target.value))}>
            {servers.map(s => <option key={s.id} value={s.id}>{s.name}</option>)}
          </select>
        </div>
      )}

      {msg && (
        <div style={{ padding: '8px 16px', background: msg.includes('失败') ? 'rgba(239,68,68,0.12)' : 'rgba(34,197,94,0.12)', border: `1px solid ${msg.includes('失败') ? 'rgba(239,68,68,0.3)' : 'rgba(34,197,94,0.3)'}`, borderRadius: 'var(--radius)', color: msg.includes('失败') ? '#ef4444' : '#22c55e', fontSize: 13, fontWeight: 500 }}>
          {msg}
        </div>
      )}

      {/* 封禁表单 */}
      <div className="card">
        <div className="card-header"><div className="card-title">手动封禁</div><div className="card-sub">输入 SteamID64 即可封禁离线玩家</div></div>
        <div className="card-body">
          <div style={{ display: 'flex', flexDirection: 'column', gap: 16, maxWidth: 520 }}>
            <div style={{ display: 'flex', gap: 8, alignItems: 'flex-end' }}>
              <div style={{ flex: 1 }}>
                <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>SteamID64</label>
                <input className="rcon-input" placeholder="7656119xxxxxxxxxx" value={steamId}
                  onChange={e => { setSteamId(e.target.value); setLookupName(''); }} />
              </div>
              <button className="rcon-btn" style={{ width: 'auto', padding: '8px 14px', fontSize: 12, whiteSpace: 'nowrap' }}
                onClick={handleLookup} disabled={looking || steamId.length < 10}>
                {looking ? '查询中...' : '查询名称'}
              </button>
            </div>
            {lookupName && (
              <div style={{ padding: '8px 12px', background: 'rgba(59,130,246,0.08)', borderRadius: 6, border: '1px solid rgba(59,130,246,0.15)', fontSize: 13 }}>
                Steam 名称：<strong>{lookupName}</strong>
                <span style={{ fontSize: 11, color: 'var(--text3)', marginLeft: 8 }}>（Steam 名称可随意更改，仅供参考）</span>
              </div>
            )}
            <div>
              <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>封禁时长</label>
              <select className="rcon-input" style={{ width: 200 }} value={duration}
                onChange={e => setDuration(parseInt(e.target.value))}>
                <option value={0}>永久封禁</option>
                <option value={60}>1 小时</option>
                <option value={360}>6 小时</option>
                <option value={1440}>1 天</option>
                <option value={10080}>7 天</option>
                <option value={43200}>30 天</option>
              </select>
              <p style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>永久封禁=0分钟，其他选项为封禁分钟数。写入 ban.cfg 格式：Ban=SteamID64:分钟数:理由</p>
            </div>
            <div>
              <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 6 }}>封禁理由</label>
              <input className="rcon-input" placeholder="Cheating / TK / Abuse / Admin decision" value={reason}
                onChange={e => setReason(e.target.value)} />
            </div>
            <button className="rcon-btn" style={{ width: 'auto', padding: '10px 24px', alignSelf: 'flex-start' }}
              onClick={handleBan} disabled={banning || !steamId || !reason}>
              {banning ? '封禁中...' : '确认封禁'}
            </button>
          </div>
        </div>
      </div>

      {/* 封禁列表 */}
      <div className="card">
        <div className="card-header">
          <div>
            <div className="card-title">封禁列表</div>
            <div className="card-sub">最终封禁列表 = RCON 远程封禁 + ban.cfg 文件封禁（共 {bans.length} 条）</div>
          </div>
          <button className="rcon-btn" style={{ width: 'auto', padding: '6px 12px', fontSize: 12 }}
            onClick={fetchBans}>刷新</button>
        </div>
        <div className="card-body" style={{ padding: 0 }}>
          {loading ? <div style={{ padding: 40, textAlign: 'center', color: 'var(--text3)' }}>加载中...</div>
          : bans.length === 0 ? <div className="empty-state"><h3>暂无封禁记录</h3></div>
          : <div style={{ overflowX: 'auto' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
              <thead><tr style={{ borderBottom: '2px solid var(--border)', textAlign: 'left' }}>
                <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>玩家名称</th>
                <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>SteamID64</th>
                <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>封禁时长</th>
                <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>封禁理由</th>
                <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>来源</th>
              </tr></thead>
              <tbody>{bans.map(b => (
                <tr key={b.steam_id} style={{ borderBottom: '1px solid var(--border)' }}>
                  <td style={{ padding: '8px 14px', fontWeight: 500 }}>{b.player_name || '-'}</td>
                  <td style={{ padding: '8px 14px', fontFamily: 'monospace', fontSize: 12, color: 'var(--text2)' }}>{b.steam_id}</td>
                  <td style={{ padding: '8px 14px' }}>
                    <span className={`badge ${b.duration === '永久封禁' ? 'red' : 'gray'}`} style={{ fontSize: 11 }}>
                      {b.duration}
                    </span>
                  </td>
                  <td style={{ padding: '8px 14px', fontSize: 12 }}>{b.reason || '-'}</td>
                  <td style={{ padding: '8px 14px' }}>
                    <span className={`badge ${b.source === 'RCON' ? 'blue' : 'gray'}`} style={{ fontSize: 10 }}>{b.source}</span>
                  </td>
                </tr>
              ))}</tbody>
            </table>
          </div>}
        </div>
      </div>
    </div>
  );
}
