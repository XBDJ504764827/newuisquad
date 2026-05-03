'use client';

import { useState, useEffect } from 'react';

const API_BASE = '/api/v1';

const LOG_TYPE_TABS = [
  { id: '', label: '全部' },
  { id: 'backend', label: '后端日志' },
  { id: 'agent', label: 'Agent日志' },
  { id: 'action', label: '操作审计' },
];

export default function ActionLogsPage() {
  const [logType, setLogType] = useState('');
  const [logs, setLogs] = useState<any[]>([]);
  const [page, setPage] = useState(1);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    setLoading(true); setPage(1);
    const qs = logType ? `?log_type=${logType}` : '';
    fetch(`${API_BASE}/operation-logs${qs}`).then(r => r.json())
      .then(d => { setLogs(d.data || []); setTotal(d.total || 0); setLoading(false); })
      .catch(() => setLoading(false));
  }, [logType]);

  useEffect(() => {
    if (page === 1) return;
    setLoading(true);
    const qs = logType ? `?log_type=${logType}&page=${page}` : `?page=${page}`;
    fetch(`${API_BASE}/operation-logs${qs}`).then(r => r.json())
      .then(d => { setLogs(d.data || []); setLoading(false); })
      .catch(() => setLoading(false));
  }, [page]);

  const levelClass = (level: string) => {
    switch (level) {
      case 'ERROR': return 'badge red';
      case 'WARN': case 'WARNING': return { backgroundColor: '#f59e0b', color: '#000', padding: '1px 6px', borderRadius: 4, fontSize: 9 };
      case 'SUCCESS': return 'badge green';
      default: return 'badge gray';
    }
  };

  return (
    <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      <div className="card">
        <div className="card-header">
          <div><div className="card-title">操作日志</div><div className="card-sub">后端系统日志 · Agent运行日志 · 管理操作审计（共 {total} 条）</div></div>
        </div>
        <div style={{ display: 'flex', gap: 4, padding: '8px 14px', borderBottom: '1px solid var(--border)' }}>
          {LOG_TYPE_TABS.map(t => (
            <button key={t.id} className={`tab-btn${logType === t.id ? ' active' : ''}`} style={{ fontSize: 12 }}
              onClick={() => { setLogType(t.id); setPage(1); }}>{t.label}</button>
          ))}
        </div>
        <div className="card-body" style={{ padding: 0 }}>
          {loading ? <div style={{ padding: 40, textAlign: 'center', color: 'var(--text3)' }}>加载中...</div>
          : logs.length === 0 ? <div className="empty-state"><h3>暂无日志</h3></div>
          : <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
            <thead><tr style={{ borderBottom: '2px solid var(--border)', textAlign: 'left' }}>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500, whiteSpace: 'nowrap' }}>时间</th>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500, width: 60 }}>类型</th>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500, width: 60 }}>级别</th>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500, width: 100 }}>模块</th>
              <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>消息</th>
            </tr></thead>
            <tbody>{logs.map((l: any, i: number) => (
              <tr key={i} style={{ borderBottom: '1px solid var(--border)' }}>
                <td style={{ padding: '6px 14px', whiteSpace: 'nowrap', fontSize: 12 }}>
                  {new Date(l.logged_at).toLocaleString()}
                </td>
                <td style={{ padding: '6px 14px' }}>
                  <span className={l.log_type === 'agent' ? 'badge blue' : l.log_type === 'action' ? 'badge green' : 'badge gray'} style={{ fontSize: 10 }}>
                    {l.log_type === 'agent' ? 'Agent' : l.log_type === 'action' ? '操作' : l.log_type === 'backend' ? '后端' : l.log_type}
                  </span>
                </td>
                <td style={{ padding: '6px 14px' }}>
                  <span className={typeof levelClass(l.level) === 'string' ? levelClass(l.level) as string : 'badge gray'} style={typeof levelClass(l.level) === 'object' ? levelClass(l.level) as any : {}}>{l.level || 'INFO'}</span>
                </td>
                <td style={{ padding: '6px 14px', fontSize: 12, color: 'var(--text2)' }}>{l.module || l.category || '-'}</td>
                <td style={{ padding: '6px 14px', fontSize: 13 }}>
                  <div>{l.message}</div>
                  {l.detail && <div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 2 }}>{l.detail}</div>}
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
