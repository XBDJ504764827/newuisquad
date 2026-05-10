'use client';

import { useState, useEffect, useCallback } from 'react';
import { api } from '../lib/api';

interface Execution {
  id: string; execution_id?: string; workflow_id: string; status: string;
  started_at: string; completed_at?: string; trigger_data?: any;
  error?: string; completed_steps?: number; failed_steps?: number; skipped_steps?: number;
}

interface Props { workflowId: string; serverId: number; }

function fmtMs(ms: number): string {
  if (ms < 1000) return `${ms.toFixed(0)}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  if (ms < 3600000) return `${(ms / 60000).toFixed(1)}m`;
  return `${(ms / 3600000).toFixed(1)}h`;
}

function fmtDate(ts: string): string {
  const d = new Date(ts);
  const today = new Date();
  if (d.toDateString() === today.toDateString()) return '今天';
  const yesterday = new Date(today); yesterday.setDate(yesterday.getDate() - 1);
  if (d.toDateString() === yesterday.toDateString()) return '昨天';
  return d.toLocaleDateString(undefined, { weekday: 'long', month: 'long', day: 'numeric' });
}

function fmtTime(ts: string): string { return new Date(ts).toLocaleTimeString(); }

function getStatus(s: string): { icon: string; color: string; bg: string; border: string; label: string } {
  switch (s.toLowerCase()) {
    case 'completed': return { icon: '✓', color: '#22c55e', bg: '#22c55e', border: '#22c55e', label: '已完成' };
    case 'failed': case 'error': return { icon: '✕', color: '#ef4444', bg: '#ef4444', border: '#ef4444', label: '失败' };
    case 'running': case 'executing': return { icon: '▶', color: '#3b82f6', bg: '#3b82f6', border: '#3b82f6', label: '执行中' };
    case 'pending': return { icon: '○', color: '#9ca3af', bg: '#9ca3af', border: '#9ca3af', label: '待执行' };
    default: return { icon: '?', color: '#9ca3af', bg: '#9ca3af', border: '#9ca3af', label: s };
  }
}

export default function WorkflowExecutionTimeline({ workflowId, serverId }: Props) {
  const [executions, setExecutions] = useState<Execution[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState('all');
  const [expanded, setExpanded] = useState<Set<string>>(new Set());

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await api(`/servers/${serverId}/workflows/${workflowId}/executions`);
      const data = await res.json();
      setExecutions(data.data || data.executions || []);
    } catch (e: any) { setError(e.message); }
    setLoading(false);
  }, [serverId, workflowId]);

  useEffect(() => { load(); }, [load]);

  const filtered = executions.filter(e => filter === 'all' || e.status.toLowerCase() === filter);
  const total = executions.length;
  const completed = executions.filter(e => e.status.toLowerCase() === 'completed').length;
  const running = executions.filter(e => e.status.toLowerCase() === 'running' || e.status.toLowerCase() === 'executing').length;
  const successRate = total > 0 ? Math.round((completed / total) * 100) : 0;

  // Group by date
  const grouped: Record<string, Execution[]> = {};
  filtered.forEach(e => {
    const date = fmtDate(e.started_at);
    if (!grouped[date]) grouped[date] = [];
    grouped[date].push(e);
  });

  const toggle = (id: string) => {
    const next = new Set(expanded);
    next.has(id) ? next.delete(id) : next.add(id);
    setExpanded(next);
  };

  const styles = {
    container: { padding: 4 },
    statsBar: { display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(150px, 1fr))', gap: 10, marginBottom: 16 },
    statCard: { background: 'var(--bg2)', borderRadius: 8, border: '1px solid var(--border)', padding: '12px 16px', textAlign: 'center' as const },
    filters: { display: 'flex', gap: 6, marginBottom: 16, flexWrap: 'wrap' as const },
    pill: (active: boolean, color?: string) => ({ padding: '4px 12px', borderRadius: 14, border: '1px solid var(--border)', background: active ? (color || 'var(--accent)') : 'transparent', color: active ? 'var(--bg)' : 'var(--text2)', cursor: 'pointer', fontSize: 12 }),
    card: (border: string) => ({ background: 'var(--bg2)', borderRadius: 8, border: `1px solid var(--border)`, borderLeft: `4px solid ${border}`, padding: '14px 16px', marginBottom: 10, cursor: 'pointer' }),
    dot: (bg: string) => ({ width: 8, height: 8, borderRadius: 4, background: bg, flexShrink: 0 }),
    badge: (bg: string, color: string) => ({ display: 'inline-block', padding: '2px 8px', borderRadius: 10, fontSize: 10, fontWeight: 600, background: bg, color }),
  };

  return (
    <div style={styles.container}>
      {/* Summary Stats */}
      <div style={styles.statsBar}>
        <div style={styles.statCard}>
          <div style={{ fontSize: 24, fontWeight: 700 }}>{total}</div>
          <div style={{ fontSize: 11, color: 'var(--text3)' }}>总执行次数</div>
        </div>
        <div style={styles.statCard}>
          <div style={{ fontSize: 24, fontWeight: 700, color: '#22c55e' }}>{successRate}%</div>
          <div style={{ fontSize: 11, color: 'var(--text3)' }}>成功率</div>
        </div>
        <div style={styles.statCard}>
          <div style={{ fontSize: 24, fontWeight: 700, color: '#3b82f6' }}>{running}</div>
          <div style={{ fontSize: 11, color: 'var(--text3)' }}>当前运行中</div>
        </div>
      </div>

      {/* Filters */}
      <div style={styles.filters}>
        {['all', 'completed', 'running', 'failed'].map(f => (
          <button key={f} onClick={() => setFilter(f)} style={styles.pill(filter === f)}>
            {f === 'all' ? '全部' : f === 'completed' ? '已完成' : f === 'running' ? '执行中' : '失败'}
          </button>
        ))}
        <button onClick={load} disabled={loading} style={{ ...styles.pill(false), marginLeft: 'auto' }}>
          {loading ? '刷新中...' : '刷新'}
        </button>
      </div>

      {error && <div style={{ padding: 10, background: 'rgba(239,68,68,0.1)', color: '#ef4444', borderRadius: 6, marginBottom: 12, fontSize: 13 }}>{error}</div>}

      {/* Timeline */}
      {loading && executions.length === 0 && <div style={{ textAlign: 'center', padding: 30, color: 'var(--text3)' }}>加载中...</div>}
      {!loading && filtered.length === 0 && <div style={{ textAlign: 'center', padding: 30, color: 'var(--text3)' }}>暂无执行记录</div>}

      {Object.entries(grouped).map(([date, exes]) => {
        const st = getStatus('');
        return (
          <div key={date} style={{ marginBottom: 20 }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 10 }}>
              <span style={{ fontSize: 12, fontWeight: 600, color: 'var(--text2)' }}>{date}</span>
              <div style={{ flex: 1, height: 1, background: 'var(--border)' }} />
            </div>
            <div>
              {exes.map(ex => {
                const status = getStatus(ex.status);
                const isOpen = expanded.has(ex.id);
                return (
                  <div key={ex.id} style={styles.card(status.border)} onClick={() => toggle(ex.id)}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 10, flexWrap: 'wrap' }}>
                      <span style={styles.dot(status.bg)} />
                      <span style={styles.badge(`rgba(${status.color === '#22c55e' ? '34,197,94' : status.color === '#ef4444' ? '239,68,68' : '59,130,246'},0.15)`, status.color)}>{status.label}</span>
                      <code style={{ fontSize: 10, color: 'var(--text3)' }}>{(ex.execution_id || ex.id).substring(0, 8)}</code>
                      <span style={{ fontSize: 11, color: 'var(--text3)' }}>{fmtTime(ex.started_at)}</span>
                      <div style={{ marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: 8 }}>
                        {ex.completed_steps != null && <span style={{ fontSize: 11, color: '#22c55e' }}>{ex.completed_steps} 完成</span>}
                        {ex.failed_steps != null && ex.failed_steps > 0 && <span style={{ fontSize: 11, color: '#ef4444' }}>{ex.failed_steps} 失败</span>}
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"
                          style={{ transform: isOpen ? 'rotate(180deg)' : 'rotate(0deg)', transition: 'transform 0.2s' }}><path d="m6 9 6 6 6-6"/></svg>
                      </div>
                    </div>
                    {isOpen && (
                      <div style={{ marginTop: 12, paddingTop: 12, borderTop: '1px solid var(--border)' }}>
                        {ex.error && (
                          <div style={{ padding: 8, background: 'rgba(239,68,68,0.1)', borderRadius: 6, marginBottom: 8, fontSize: 12, color: '#ef4444' }}>{ex.error}</div>
                        )}
                        {ex.trigger_data && (
                          <div>
                            <div style={{ fontSize: 11, color: 'var(--text3)', marginBottom: 4 }}>触发数据</div>
                            <pre style={{ background: 'var(--bg)', padding: 8, borderRadius: 6, fontSize: 11, color: 'var(--text)', overflow: 'auto', maxHeight: 150, margin: 0 }}>{JSON.stringify(ex.trigger_data, null, 2)}</pre>
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        );
      })}
    </div>
  );
}
