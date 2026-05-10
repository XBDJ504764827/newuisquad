'use client';

import { useState, useEffect, useCallback } from 'react';
import { useServers } from '../../lib/useServers';
import { api } from '../../lib/api';
import Pagination from '../Pagination';

interface AuditStats {
  period_days: number; rcon_commands: number; admin_actions: number;
  chat_violations: number; system_errors: number; unique_admins: number;
  action_breakdown: { action: string; count: number }[];
  daily_trend: { date: string; count: number }[];
}

const ACTION_COLORS: Record<string, string> = { kick: '#ef4444', ban: '#dc2626', warn: '#f59e0b', broadcast: '#3b82f6', RCON: '#8b5cf6', change_layer: '#10b981' };
const actionLabel = (a: string) => ({ warn: '警告', kick: '踢出', ban: '封禁', broadcast: '广播', change_layer: '切换地图', RCON: 'RCON命令' }[a] || a);
const levelBadge = (level: string) => {
  switch (level) {
    case 'ERROR': return <span style={{ padding: '1px 6px', borderRadius: 4, fontSize: 9, background: 'rgba(239,68,68,0.15)', color: '#ef4444' }}>ERROR</span>;
    case 'WARN': case 'WARNING': return <span style={{ padding: '1px 6px', borderRadius: 4, fontSize: 9, background: 'rgba(245,158,11,0.15)', color: '#f59e0b' }}>WARN</span>;
    case 'SUCCESS': return <span style={{ padding: '1px 6px', borderRadius: 4, fontSize: 9, background: 'rgba(34,197,94,0.15)', color: '#22c55e' }}>OK</span>;
    default: return <span style={{ padding: '1px 6px', borderRadius: 4, fontSize: 9, background: 'rgba(156,163,175,0.15)', color: 'var(--text3)' }}>INFO</span>;
  }
};

export default function AuditDashboardPage() {
  const { servers } = useServers();
  const [activeTab, setActiveTab] = useState<'dashboard' | 'logs'>('dashboard');

  // Dashboard state
  const [stats, setStats] = useState<AuditStats | null>(null);
  const [entries, setEntries] = useState<any[]>([]);
  const [dLoading, setDLoading] = useState(true);
  const [days, setDays] = useState(7);

  // Logs state
  const [logType, setLogType] = useState('');
  const [serverId, setServerId] = useState<number | null>(null);
  const [logs, setLogs] = useState<any[]>([]);
  const [page, setPage] = useState(1);
  const [total, setTotal] = useState(0);
  const [lLoading, setLLoading] = useState(false);

  useEffect(() => { if (servers.length > 0 && !serverId) setServerId(servers[0].id); }, [servers, serverId]);

  // Dashboard load
  useEffect(() => {
    if (activeTab !== 'dashboard') return;
    setDLoading(true);
    Promise.all([
      api(`/audit-stats?date_from=${days}`).then(r => r.ok ? r.json() : null),
      api('/audit-detail?per_page=100').then(r => r.ok ? r.json() : null),
    ]).then(([s, e]) => {
      if (s) setStats(s);
      if (e) setEntries(e.data || []);
    }).finally(() => setDLoading(false));
  }, [days, activeTab]);

  // Logs load
  useEffect(() => {
    if (activeTab !== 'logs') return;
    setLLoading(true);
    const params = new URLSearchParams();
    if (logType) params.set('log_type', logType);
    if (logType === 'action' && serverId) params.set('server_id', String(serverId));
    params.set('page', String(page));
    const qs = params.toString();
    api(`/operation-logs${qs ? '?' + qs : ''}`)
      .then(r => r.json())
      .then(d => { setLogs(d.data || []); setTotal(d.total || 0); })
      .catch(() => {})
      .finally(() => setLLoading(false));
  }, [logType, serverId, page, activeTab]);

  const maxTrend = stats ? Math.max(...stats.daily_trend.map(d => d.count), 1) : 1;

  const s = {
    tabs: { display: 'flex', gap: 2, marginBottom: 20, borderBottom: '1px solid var(--border)' },
    tab: (active: boolean) => ({ padding: '10px 20px', border: 'none', background: active ? 'var(--text)' : 'transparent', color: active ? 'var(--bg)' : 'var(--text2)', cursor: 'pointer', borderRadius: '8px 8px 0 0', fontSize: 13, fontWeight: active ? 600 : 400 }),
    card: { background: 'var(--bg2)', borderRadius: 10, border: '1px solid var(--border)', padding: 16, marginBottom: 12 },
    statCard: { background: 'var(--bg2)', borderRadius: 10, padding: '14px 16px', border: '1px solid var(--border)', minWidth: 130 },
    th: { padding: '8px 12px', textAlign: 'left' as const, fontWeight: 500, color: 'var(--text3)', fontSize: 12, borderBottom: '2px solid var(--border)' },
    td: { padding: '8px 12px', color: 'var(--text2)', fontSize: 12, borderBottom: '1px solid var(--border)' },
  };

  return (
    <div style={{ padding: 20 }}>
      <h2 style={{ margin: '0 0 16px 0', fontSize: 20, fontWeight: 600 }}>操作审计</h2>

      <div style={s.tabs}>
        <button onClick={() => setActiveTab('dashboard')} style={s.tab(activeTab === 'dashboard')}>统计仪表盘</button>
        <button onClick={() => setActiveTab('logs')} style={s.tab(activeTab === 'logs')}>操作日志</button>
      </div>

      {/* ===== Dashboard Tab ===== */}
      {activeTab === 'dashboard' && (
        <>
          <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 16 }}>
            <span style={{ fontSize: 12, color: 'var(--text3)' }}>时间范围：</span>
            <select value={days} onChange={e => setDays(Number(e.target.value))}
              style={{ padding: '4px 12px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--bg2)', color: 'var(--text)' }}>
              <option value={1}>24小时</option><option value={7}>7天</option><option value={30}>30天</option>
            </select>
          </div>
          {dLoading && <div style={{ textAlign: 'center', padding: 40, color: 'var(--text3)' }}>加载中...</div>}
          {stats && !dLoading && (
            <>
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(150px, 1fr))', gap: 12, marginBottom: 16 }}>
                {[{ label: 'RCON命令', value: stats.rcon_commands, color: '#8b5cf6' }, { label: '管理操作', value: stats.admin_actions, color: '#3b82f6' }, { label: '聊天违规', value: stats.chat_violations, color: '#ef4444' }, { label: '系统错误', value: stats.system_errors, color: '#f59e0b' }, { label: '活跃管理员', value: stats.unique_admins, color: '#10b981' }].map(c => (
                  <div key={c.label} style={s.statCard}>
                    <div style={{ fontSize: 12, color: 'var(--text3)', marginBottom: 6 }}>{c.label}</div>
                    <div style={{ fontSize: 28, fontWeight: 700, color: c.color }}>{c.value}</div>
                  </div>
                ))}
              </div>
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 2fr', gap: 12, marginBottom: 16 }}>
                <div style={s.card}>
                  <div style={{ fontSize: 14, fontWeight: 600, marginBottom: 12 }}>操作分类</div>
                  {stats.action_breakdown.map(item => (
                    <div key={item.action} style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
                      <span style={{ width: 8, height: 8, borderRadius: 4, background: ACTION_COLORS[item.action] || '#6b7280' }} />
                      <span style={{ flex: 1, fontSize: 13, color: 'var(--text2)' }}>{actionLabel(item.action)}</span>
                      <span style={{ fontSize: 13, fontWeight: 600, color: 'var(--text)' }}>{item.count}</span>
                    </div>
                  ))}
                </div>
                <div style={s.card}>
                  <div style={{ fontSize: 14, fontWeight: 600, marginBottom: 12 }}>每日操作趋势</div>
                  <div style={{ display: 'flex', alignItems: 'flex-end', gap: 4, height: 120 }}>
                    {stats.daily_trend.map((d, i) => (
                      <div key={i} style={{ flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 4 }}>
                        <span style={{ fontSize: 10, color: 'var(--text2)' }}>{d.count}</span>
                        <div style={{ width: '100%', maxWidth: 30, height: Math.max((d.count / maxTrend) * 100, 4), background: 'var(--text)', borderRadius: '4px 4px 0 0', opacity: 0.7 + (d.count / maxTrend) * 0.3 }} />
                        <span style={{ fontSize: 9, color: 'var(--text3)' }}>{d.date.slice(5)}</span>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
              {/* Recent entries */}
              <div style={{ ...s.card, padding: 0, overflow: 'hidden' }}>
                <div style={{ padding: '12px 16px', fontSize: 14, fontWeight: 600, borderBottom: '1px solid var(--border)' }}>最近操作</div>
                <div style={{ maxHeight: 400, overflow: 'auto' }}>
                  <table style={{ width: '100%', borderCollapse: 'collapse' }}>
                    <thead><tr>
                      <th style={s.th}>时间</th><th style={s.th}>管理员</th><th style={s.th}>操作</th><th style={s.th}>目标</th><th style={s.th}>消息</th>
                    </tr></thead>
                    <tbody>
                      {entries.slice(0, 50).map((e, i) => (
                        <tr key={i}>
                          <td style={s.td}>{new Date(e.logged_at).toLocaleString('zh-CN')}</td>
                          <td style={s.td}>{e.admin_user}</td>
                          <td style={s.td}><span style={{ color: ACTION_COLORS[e.action_type] || '#6b7280' }}>{actionLabel(e.action_type)}</span></td>
                          <td style={s.td}>{(e.target || '').slice(0, 40)}</td>
                          <td style={s.td}>{e.message || '-'}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            </>
          )}
        </>
      )}

      {/* ===== Logs Tab ===== */}
      {activeTab === 'logs' && (
        <>
          <div style={{ display: 'flex', gap: 8, alignItems: 'center', marginBottom: 12, flexWrap: 'wrap' }}>
            <div style={{ display: 'flex', gap: 4 }}>
              {[{ id: '', label: '全部' }, { id: 'backend', label: '后端' }, { id: 'agent', label: 'Agent' }, { id: 'action', label: '操作' }].map(t => (
                <button key={t.id} onClick={() => { setLogType(t.id); setPage(1); }}
                  style={{ padding: '5px 14px', borderRadius: 14, border: '1px solid var(--border)', background: logType === t.id ? 'var(--text)' : 'transparent', color: logType === t.id ? 'var(--bg)' : 'var(--text2)', cursor: 'pointer', fontSize: 12 }}>{t.label}</button>
              ))}
            </div>
            {logType === 'action' && (
              <>
                <span style={{ fontSize: 12, color: 'var(--text3)' }}>服务器：</span>
                <select style={{ padding: '5px 10px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--bg2)', color: 'var(--text)', fontSize: 12 }}
                  value={serverId || ''} onChange={e => { setServerId(e.target.value ? parseInt(e.target.value) : null); setPage(1); }}>
                  <option value="">全部</option>
                  {servers.map(s => <option key={s.id} value={s.id}>{s.name}</option>)}
                </select>
              </>
            )}
            <span style={{ fontSize: 11, color: 'var(--text3)' }}>共 {total} 条</span>
          </div>

          {lLoading ? <div style={{ textAlign: 'center', padding: 40, color: 'var(--text3)' }}>加载中...</div> :
            logs.length === 0 ? <div style={{ textAlign: 'center', padding: 40, color: 'var(--text3)' }}>暂无日志记录</div> :
            <div style={{ ...s.card, padding: 0, overflow: 'hidden' }}>
              <div style={{ overflow: 'auto' }}>
                <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
                  <thead><tr>
                    <th style={s.th}>时间</th>
                    <th style={s.th}>级别</th>
                    {logType === 'action' && <th style={s.th}>来源</th>}
                    <th style={s.th}>模块</th>
                    {logType === 'action' && <th style={s.th}>操作者</th>}
                    <th style={s.th}>消息</th>
                  </tr></thead>
                  <tbody>
                    {logs.map((l: any, i: number) => (
                      <tr key={l.id || i} style={{ borderBottom: '1px solid var(--border)' }}>
                        <td style={{ ...s.td, whiteSpace: 'nowrap' }}>{new Date(l.logged_at).toLocaleString()}</td>
                        <td style={s.td}>{levelBadge(l.level)}</td>
                        {logType === 'action' && <td style={s.td}>{l.source}</td>}
                        <td style={s.td}>{l.module || l.category || '-'}</td>
                        {logType === 'action' && <td style={s.td}>{l.admin_user || '-'}</td>}
                        <td style={s.td}>{l.message}{l.detail && <div style={{ fontSize: 10, color: 'var(--text3)', marginTop: 2, whiteSpace: 'pre-wrap', wordBreak: 'break-all' }}>{l.detail}</div>}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                <Pagination page={page} total={total} perPage={50} onPageChange={setPage} />
              </div>
            </div>
          }
        </>
      )}
    </div>
  );
}
