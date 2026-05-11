'use client';

import { useState, useEffect, useCallback } from 'react';

import { useServers } from '../../lib/useServers';
import { api } from '../../lib/api';
import type { KillEvent } from '../../types';
import Pagination from '../Pagination';
import TimeRangeFilter from '../TimeRangeFilter';

const EVENT_TYPE_LABELS: Record<string, { label: string; className: string }> = {
  damage: { label: '伤害', className: 'gray' },
  wound: { label: '击倒', className: 'green' },
  death: { label: '阵亡', className: 'red' },
};

export default function KillLogsPage() {
  const { servers } = useServers();
  const [serverId, setServerId] = useState<number | null>(null);
  const [events, setEvents] = useState<KillEvent[]>([]);
  const [page, setPage] = useState(1);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(false);
  const [appliedStart, setAppliedStart] = useState('');
  const [appliedEnd, setAppliedEnd] = useState('');

  useEffect(() => {
    if (servers.length > 0 && !serverId) setServerId(servers[0].id);
  }, [servers, serverId]);

  const load = useCallback(() => {
    if (!serverId) return;
    setLoading(true);
    const params = new URLSearchParams();
    params.set('page', String(page));
    params.set('per_page', '50');
    if (appliedStart) params.set('start', new Date(appliedStart).toISOString());
    if (appliedEnd) params.set('end', new Date(appliedEnd).toISOString());
    api(`/servers/${serverId}/kill-events?${params.toString()}`).then(r => r.json())
      .then(d => { setEvents(d.data || []); setTotal(d.total || 0); setLoading(false); }).catch(() => setLoading(false));
  }, [serverId, page, appliedStart, appliedEnd]);

  useEffect(() => { load(); }, [load]);

  const typeInfo = (e: KillEvent) => {
    if (e.is_teamkill) return { label: 'TK', className: 'red' };
    return EVENT_TYPE_LABELS[e.event_type] || { label: e.event_type, className: 'gray' };
  };

  return (
    <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      <div style={{ display: 'flex', gap: 8, alignItems: 'center', flexWrap: 'wrap' }}>
        <span style={{ fontSize: 12, color: 'var(--text3)' }}>服务器：</span>
        <select className="rcon-input" style={{ width: 'auto', padding: '6px 10px' }} value={serverId || ''}
          onChange={e => { setServerId(parseInt(e.target.value)); setPage(1); }}>
          {servers.map(s => <option key={s.id} value={s.id}>{s.name}</option>)}
        </select>
        <TimeRangeFilter
          onApply={(s, e) => { setAppliedStart(s); setAppliedEnd(e); setPage(1); }}
          onClear={() => { setAppliedStart(''); setAppliedEnd(''); setPage(1); }}
          hasFilter={!!(appliedStart || appliedEnd)}
        />
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
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>武器</th>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>伤害值</th>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>受害者</th>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>类型</th>
            </tr></thead>
            <tbody>{events.map(e => {
              const ti = typeInfo(e);
              return (
              <tr key={e.id} style={{ borderBottom: '1px solid var(--border)' }}>
                <td style={{ padding: '8px 14px', whiteSpace: 'nowrap', fontSize: 12 }}>{new Date(e.logged_at).toLocaleString()}</td>
                <td style={{ padding: '8px 14px', fontWeight: 500 }}>
                  {e.attacker_name || e.attacker_steam64 || '-'}
                  {e.attacker_steam64 && <span style={{ fontFamily: 'monospace', fontSize: 10, color: 'var(--text3)', marginLeft: 4 }}>{e.attacker_steam64}</span>}
                </td>
                <td style={{ padding: '8px 14px' }}><span className="badge blue" style={{ fontSize: 10 }}>{e.weapon}</span></td>
                <td style={{ padding: '8px 14px', color: e.is_teamkill ? 'var(--red)' : e.event_type === 'death' ? '#ef4444' : '#f59e0b', fontWeight: 600 }}>{e.damage.toFixed(1)}</td>
                <td style={{ padding: '8px 14px', fontWeight: 500 }}>
                  {e.victim_name}
                  {e.victim_steam64 && <span style={{ fontFamily: 'monospace', fontSize: 10, color: 'var(--text3)', marginLeft: 4 }}>{e.victim_steam64}</span>}
                </td>
                <td style={{ padding: '8px 14px' }}>
                  <span className={`badge ${ti.className}`} style={{ fontSize: 10 }}>{ti.label}</span>
                </td>
              </tr>
            )})}</tbody>
          </table>}
          <Pagination page={page} total={total} perPage={50} onPageChange={setPage} />
        </div>
      </div>
    </div>
  );
}
