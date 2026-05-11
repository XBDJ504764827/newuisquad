'use client';

import { useState, useEffect, useCallback } from 'react';

import { useServers } from '../../lib/useServers';
import { api } from '../../lib/api';
import Pagination from '../Pagination';
import TimeRangeFilter from '../TimeRangeFilter';

interface FlyEvent { id: number; player_name: string; eos_id: string; steam64: string; event_type: string; logged_at: string; }

const TYPE_LABELS: Record<string, string> = {
  possess: '进入镜头', unpossess: '退出镜头', admin_camera: '管理员镜头', spectate: '观战',
};

export default function FlyLogsPage() {
  const { servers } = useServers();
  const [serverId, setServerId] = useState<number | null>(null);
  const [events, setEvents] = useState<FlyEvent[]>([]);
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
    api(`/servers/${serverId}/fly-events?${params.toString()}`).then(r => r.json())
      .then(d => { setEvents(d.data || []); setTotal(d.total || 0); setLoading(false); }).catch(() => setLoading(false));
  }, [serverId, page, appliedStart, appliedEnd]);

  useEffect(() => { load(); }, [load]);

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
                <td style={{ padding: '8px 14px', whiteSpace: 'nowrap', fontSize: 12 }}>{new Date(e.logged_at).toLocaleString()}</td>
                <td style={{ padding: '8px 14px', fontWeight: 500 }}>{e.player_name || '-'}</td>
                <td style={{ padding: '8px 14px', fontFamily: 'monospace', fontSize: 12, color: 'var(--text2)' }}>{e.steam64 || '-'}</td>
                <td style={{ padding: '8px 14px', fontFamily: 'monospace', fontSize: 11, color: 'var(--text2)' }}>{e.eos_id || '-'}</td>
                <td style={{ padding: '8px 14px' }}><span className={`badge ${e.event_type === 'possess' || e.event_type === 'admin_camera' ? 'green' : 'gray'}`} style={{ fontSize: 11 }}>{TYPE_LABELS[e.event_type] || e.event_type}</span></td>
              </tr>
            ))}</tbody>
          </table>}
          <Pagination page={page} total={total} perPage={50} onPageChange={setPage} />
        </div>
      </div>
    </div>
  );
}
