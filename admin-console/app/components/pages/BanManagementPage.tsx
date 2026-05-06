'use client';

import { useState, useEffect, useCallback, useRef } from 'react';
import { useServers } from '../../lib/useServers';
import { api } from '../../lib/api';

interface BanEntry { steam_id: string; player_name: string; duration: string; reason: string; source: string; }

export default function BanManagementPage() {
  const { servers } = useServers();
  const [serverId, setServerId] = useState<number | null>(null);
  const [bans, setBans] = useState<BanEntry[]>([]);
  const [loading, setLoading] = useState(false);

  const [steamId, setSteamId] = useState('');
  const [playerName, setPlayerName] = useState('');
  const [reason, setReason] = useState('');
  const [duration, setDuration] = useState('perm');
  const [lookupName, setLookupName] = useState('');
  const [looking, setLooking] = useState(false);
  const [banning, setBanning] = useState(false);
  const [msg, setMsg] = useState('');
  const [showModal, setShowModal] = useState(false);
  const overlayRef = useRef<HTMLDivElement>(null);

  useEffect(() => { if (servers.length > 0 && !serverId) setServerId(servers[0].id); }, [servers, serverId]);

  const fetchBans = useCallback(() => {
    if (!serverId) return; setLoading(true);
    api(`/servers/${serverId}/ban-list`).then(r => r.json())
      .then(d => { setBans(d.data || []); setLoading(false); }).catch(() => setLoading(false));
  }, [serverId]);

  useEffect(() => { fetchBans(); }, [fetchBans]);

  const handleLookup = () => {
    if (!steamId || steamId.length < 10) return;
    setLooking(true); setLookupName('');
    api(`/steam-player/${steamId}`).then(r => r.json())
      .then(d => { setLookupName(d.player_name || ''); setPlayerName(d.player_name || ''); setLooking(false); })
      .catch(() => setLooking(false));
  };

  const getDurationMinutes = () => {
    switch (duration) {
      case '1h': return 60;
      case '6h': return 360;
      case '1d': return 1440;
      case '7d': return 10080;
      case '30d': return 43200;
      default: return 0;
    }
  };

  const getDurationLabel = () => {
    switch (duration) {
      case '1h': return '1 小时';
      case '6h': return '6 小时';
      case '1d': return '1 天';
      case '7d': return '7 天';
      case '30d': return '30 天';
      default: return '永久';
    }
  };

  const handleBan = () => {
    if (!serverId || !steamId || !reason) return;
    setBanning(true);
    api(`/servers/${serverId}/ban-player`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ steam_id: steamId, reason: `${reason} 处理人：${playerName || '管理员'}`, duration: getDurationMinutes() }),
    }).then(r => r.json()).then(d => {
      setBanning(false);
      if (d.error) { setMsg(d.error); setTimeout(() => setMsg(''), 4000); }
      else { setMsg(d.message || '封禁成功'); setSteamId(''); setReason(''); setPlayerName(''); setLookupName(''); setDuration('perm'); setShowModal(false); fetchBans(); setTimeout(() => setMsg(''), 4000); }
    }).catch(() => { setBanning(false); setMsg('操作失败'); setTimeout(() => setMsg(''), 4000); });
  };

  const closeModal = () => setShowModal(false);
  const openAddModal = () => { setSteamId(''); setPlayerName(''); setReason(''); setDuration('perm'); setLookupName(''); setShowModal(true); };

  const cfgBaseUrl = typeof window !== 'undefined' ? `${window.location.origin}/api/v1/servers` : '';

  return (
    <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      {servers.length > 0 && (
        <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
          <span style={{ fontSize: 12, color: 'var(--text3)' }}>服务器：</span>
          <select className="form-select" style={{ width: 'auto', padding: '6px 10px' }} value={serverId || ''}
            onChange={e => setServerId(parseInt(e.target.value))}>
            {servers.map(s => <option key={s.id} value={s.id}>{s.name}</option>)}
          </select>
        </div>
      )}

      {msg && (
        <div style={{ padding: '8px 16px', background: msg.includes('失败') || msg.includes('错误') ? 'rgba(239,68,68,0.12)' : 'rgba(34,197,94,0.12)', border: `1px solid ${msg.includes('失败') || msg.includes('错误') ? 'rgba(239,68,68,0.3)' : 'rgba(34,197,94,0.3)'}`, borderRadius: 'var(--radius)', color: msg.includes('失败') || msg.includes('错误') ? '#ef4444' : '#22c55e', fontSize: 13, fontWeight: 500 }}>{msg}</div>
      )}

      <div className="card">
        <div className="card-header">
          <div><div className="card-title">被封禁玩家列表</div><div className="card-sub">管理服务器黑名单及封禁记录</div></div>
          <button className="btn btn-primary" onClick={openAddModal}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
            手动添加封禁
          </button>
        </div>
        <div style={{ overflowX: 'auto' }}>
          {loading ? <div style={{ padding: 40, textAlign: 'center', color: 'var(--text3)' }}>加载中...</div>
          : bans.length === 0 ? <div className="empty-state"><h3>暂无封禁记录</h3></div>
          : <table>
            <thead>
              <tr><th>封禁时间</th><th>SteamID64</th><th>玩家名称(曾用)</th><th>封禁时长</th><th>封禁理由</th><th>操作人</th><th>状态</th></tr>
            </thead>
            <tbody>{bans.map(b => (
              <tr key={b.steam_id}>
                <td style={{ fontSize: 12 }}>{new Date().toISOString().slice(0, 10)}</td>
                <td style={{ fontFamily: 'monospace', fontSize: 12, color: 'var(--text2)' }}>{b.steam_id}</td>
                <td style={{ fontWeight: 500 }}>{b.player_name || '-'}</td>
                <td>{b.duration === '永久封禁' ? '永久' : b.duration}</td>
                <td style={{ fontSize: 12, maxWidth: 260, overflow: 'hidden', textOverflow: 'ellipsis' }} title={b.reason}>{b.reason || '-'}</td>
                <td style={{ color: 'var(--text2)' }}>{b.source}</td>
                <td><span className="badge red">生效中</span></td>
              </tr>
            ))}</tbody>
          </table>}
        </div>
      </div>

      <div className="card">
        <div className="card-header"><div><div className="card-title">Bans.cfg 下载地址</div><div className="card-sub">填入 RemoteAdminListHosts.cfg 使游戏服务器自动同步</div></div></div>
        <div className="card-body">
          {serverId && <div className="cfg-url-box"><code>{cfgBaseUrl}/{serverId}/Bans.cfg</code></div>}
          <p style={{ fontSize: 12, color: 'var(--text3)', marginTop: 8 }}>公开端点，无需认证。</p>
        </div>
      </div>

      {/* 添加封禁 Modal */}
      {showModal && (
        <div className="modal-overlay" ref={overlayRef} onClick={e => { if (e.target === overlayRef.current) closeModal(); }}>
          <div className="modal-box">
            <div className="modal-header">
              <div className="modal-title">添加玩家封禁</div>
              <button className="modal-close" onClick={closeModal}><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg></button>
            </div>
            <div className="modal-body">
              <div className="form-group">
                <label className="form-label">SteamID64 <span style={{ color: 'var(--red)' }}>*</span></label>
                <input type="text" className="form-input" placeholder="例如：76561198000000000" value={steamId}
                  onChange={e => { setSteamId(e.target.value); setLookupName(''); }} />
                <button className="btn btn-outline" style={{ alignSelf: 'flex-start', marginTop: 4, padding: '4px 10px' }}
                  onClick={handleLookup} disabled={looking || steamId.length < 10} type="button">
                  {looking ? '查询中...' : '查询 Steam 名称'}
                </button>
                {lookupName && <div style={{ fontSize: 12, color: 'var(--text2)', marginTop: 4 }}>Steam 名称：<strong>{lookupName}</strong></div>}
              </div>
              <div className="form-group">
                <label className="form-label">玩家名称 (选填，仅作备注记录)</label>
                <input type="text" className="form-input" placeholder="玩家当前或曾用名称" value={playerName}
                  onChange={e => setPlayerName(e.target.value)} />
              </div>
              <div className="form-group">
                <label className="form-label">封禁时长 <span style={{ color: 'var(--red)' }}>*</span></label>
                <select className="form-select" value={duration} onChange={e => setDuration(e.target.value)}>
                  <option value="1h">1 小时</option>
                  <option value="6h">6 小时</option>
                  <option value="1d">1 天</option>
                  <option value="7d">7 天</option>
                  <option value="30d">30 天</option>
                  <option value="perm">永久封禁</option>
                </select>
              </div>
              <div className="form-group">
                <label className="form-label">封禁理由 <span style={{ color: 'var(--red)' }}>*</span></label>
                <textarea className="form-input form-textarea" rows={3} placeholder="填写封禁的具体原因..." value={reason}
                  onChange={e => setReason(e.target.value)} />
              </div>
            </div>
            <div className="modal-footer">
              <button className="btn btn-outline" onClick={closeModal} type="button">取消</button>
              <button className="btn btn-primary" onClick={handleBan} disabled={banning || !steamId || !reason} type="button">
                {banning ? '执行中...' : '确认封禁'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
