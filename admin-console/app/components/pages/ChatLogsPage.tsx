'use client';

import { useState, useEffect, useCallback } from 'react';
import { useServers } from '../../lib/useServers';
import { api } from '../../lib/api';
import Pagination from '../Pagination';
import TimeRangeFilter from '../TimeRangeFilter';

export default function ChatLogsPage() {
  const { servers, loading: serversLoading } = useServers();
  const [serverId, setServerId] = useState<number | null>(null);
  const [messages, setMessages] = useState<any[]>([]);
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
    params.set('per_page', '100');
    if (appliedStart) params.set('start', new Date(appliedStart).toISOString());
    if (appliedEnd) params.set('end', new Date(appliedEnd).toISOString());
    api(`/servers/${serverId}/chat-messages?${params.toString()}`).then(r => r.json())
      .then(d => { setMessages(d.data || []); setTotal(d.total || 0); setLoading(false); }).catch(() => setLoading(false));
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
              <tr key={(m as any).id ?? i} style={{ borderBottom: '1px solid var(--border)' }}>
                <td style={{ padding: '6px 14px', whiteSpace: 'nowrap', fontSize: 12 }}>{new Date(m.logged_at).toLocaleString()}</td>
                <td style={{ padding: '6px 14px', fontWeight: 500 }}>{m.player_name || '-'}</td>
                <td style={{ padding: '6px 14px' }}>
                  <span className={`badge ${m.channel === 'Team' ? 'blue' : 'gray'}`} style={{ fontSize: 10 }}>{m.channel === 'Team' ? '小队' : '全局'}</span>
                </td>
                <td style={{ padding: '6px 14px' }}>{m.message}</td>
              </tr>
            ))}</tbody>
          </table>}
          <Pagination page={page} total={total} perPage={100} onPageChange={setPage} />
        </div>
      </div>
    </div>
  );
}
