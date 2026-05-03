'use client';

import { useState, useEffect } from 'react';
import { useServers } from '../../lib/useServers';
import { api } from '../../lib/api';
import Pagination from '../Pagination';

const LOG_TYPE_TABS = [
    { id: '', label: '全部' },
    { id: 'backend', label: '后端日志' },
    { id: 'agent', label: 'Agent日志' },
    { id: 'action', label: '操作审计' },
];

const SOURCE_BADGES: Record<string, { color: string; label: string }> = {
    system: { color: 'blue', label: '系统' },
    rcon: { color: 'green', label: 'RCON' },
    admin_action: { color: 'red', label: '游戏' },
    agent: { color: 'gray', label: 'Agent' },
};

export default function ActionLogsPage() {
    const { servers } = useServers();
    const [logType, setLogType] = useState('');
    const [serverId, setServerId] = useState<number | null>(null);
    const [logs, setLogs] = useState<any[]>([]);
    const [page, setPage] = useState(1);
    const [total, setTotal] = useState(0);
    const [loading, setLoading] = useState(false);

    useEffect(() => {
        if (servers.length > 0 && !serverId) setServerId(servers[0].id);
    }, [servers, serverId]);

    useEffect(() => {
        setLoading(true);
        setPage(1);
        const params = new URLSearchParams();
        if (logType) params.set('log_type', logType);
        if (logType === 'action' && serverId) params.set('server_id', String(serverId));
        const qs = params.toString();
        api(`/operation-logs${qs ? '?' + qs : ''}`)
            .then(r => r.json())
            .then(d => { setLogs(d.data || []); setTotal(d.total || 0); setLoading(false); })
            .catch(e => { console.error(e); setLoading(false); });
    }, [logType, serverId]);

    useEffect(() => {
        if (page === 1) return;
        setLoading(true);
        const params = new URLSearchParams();
        if (logType) params.set('log_type', logType);
        if (logType === 'action' && serverId) params.set('server_id', String(serverId));
        params.set('page', String(page));
        api(`/operation-logs?${params}`)
            .then(r => r.json())
            .then(d => { setLogs(d.data || []); setLoading(false); })
            .catch(e => { console.error(e); setLoading(false); });
    }, [page]);

    const levelBadge = (level: string) => {
        switch (level) {
            case 'ERROR': return <span className="badge red" style={{ fontSize: 10 }}>ERROR</span>;
            case 'WARN': case 'WARNING': return <span style={{ backgroundColor: '#f59e0b', color: '#000', padding: '1px 6px', borderRadius: 4, fontSize: 9 }}>WARN</span>;
            case 'SUCCESS': return <span className="badge green" style={{ fontSize: 10 }}>OK</span>;
            default: return <span className="badge gray" style={{ fontSize: 10 }}>INFO</span>;
        }
    };

    const sourceBadge = (source: string) => {
        const info = SOURCE_BADGES[source] || { color: 'gray', label: source };
        return <span className={`badge ${info.color}`} style={{ fontSize: 10 }}>{info.label}</span>;
    };

    const logTypeBadge = (lt: string) => {
        switch (lt) {
            case 'action': return <span className="badge green" style={{ fontSize: 10 }}>操作</span>;
            case 'agent': return <span className="badge blue" style={{ fontSize: 10 }}>Agent</span>;
            case 'backend': return <span className="badge gray" style={{ fontSize: 10 }}>后端</span>;
            case 'auth': return <span className="badge green" style={{ fontSize: 10 }}>认证</span>;
            default: return <span className="badge gray" style={{ fontSize: 10 }}>{lt}</span>;
        }
    };

    return (
        <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
            <div style={{ display: 'flex', gap: 12, alignItems: 'center', flexWrap: 'wrap' }}>
                {/* 服务器选择（操作审计模式） */}
                {logType === 'action' && (
                    <>
                        <span style={{ fontSize: 12, color: 'var(--text3)' }}>服务器：</span>
                        <select className="rcon-input" style={{ width: 'auto', padding: '6px 10px' }}
                            value={serverId || ''}
                            onChange={e => setServerId(e.target.value ? parseInt(e.target.value) : null)}>
                            <option value="">全部服务器</option>
                            {servers.map(s => <option key={s.id} value={s.id}>{s.name}</option>)}
                        </select>
                    </>
                )}
            </div>

            <div className="card">
                <div className="card-header">
                    <div>
                        <div className="card-title">
                            {logType === 'action' ? '管理员操作审计' : '操作日志'}
                        </div>
                        <div className="card-sub">
                            {logType === 'action'
                                ? 'RCON命令 · 管理操作 · 游戏内管理行为（共 ' + total + ' 条）'
                                : '后端系统日志 · Agent运行日志 · 管理操作审计（共 ' + total + ' 条）'}
                        </div>
                    </div>
                </div>

                {/* 标签切换 */}
                <div style={{ display: 'flex', gap: 4, padding: '8px 14px', borderBottom: '1px solid var(--border)' }}>
                    {LOG_TYPE_TABS.map(t => (
                        <button key={t.id}
                            className={`tab-btn${logType === t.id ? ' active' : ''}`}
                            style={{ fontSize: 12 }}
                            onClick={() => { setLogType(t.id); setPage(1); }}>
                            {t.label}
                        </button>
                    ))}
                </div>

                {/* 日志表格 */}
                <div className="card-body" style={{ padding: 0 }}>
                    {loading ? (
                        <div style={{ padding: 40, textAlign: 'center', color: 'var(--text3)' }}>加载中...</div>
                    ) : logs.length === 0 ? (
                        <div className="empty-state">
                            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
                                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                                <polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/>
                            </svg>
                            <h3>暂无日志</h3>
                            <p style={{ marginTop: 8, fontSize: 12 }}>系统将在管理操作发生时自动记录。</p>
                        </div>
                    ) : (
                        <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
                            <thead>
                                <tr style={{ borderBottom: '2px solid var(--border)', textAlign: 'left' }}>
                                    <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500, whiteSpace: 'nowrap', width: 140 }}>时间</th>
                                    {logType !== 'action' && (
                                        <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500, width: 60 }}>类型</th>
                                    )}
                                    <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500, width: 60 }}>级别</th>
                                    {logType === 'action' && (
                                        <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500, width: 70 }}>来源</th>
                                    )}
                                    <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500, width: 90 }}>模块</th>
                                    {logType === 'action' && (
                                        <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500, width: 100 }}>操作者</th>
                                    )}
                                    <th style={{ padding: '10px 14px', color: 'var(--text3)', fontWeight: 500 }}>消息</th>
                                </tr>
                            </thead>
                            <tbody>
                                {logs.map((l) => (
                                    <tr key={l.id || l.logged_at + (l.message || '').slice(0, 20)}
                                        style={{ borderBottom: '1px solid var(--border)' }}>
                                        <td style={{ padding: '6px 14px', whiteSpace: 'nowrap', fontSize: 12, color: 'var(--text2)' }}>
                                            {new Date(l.logged_at).toLocaleString()}
                                        </td>
                                        {logType !== 'action' && (
                                            <td style={{ padding: '6px 14px' }}>{logTypeBadge(l.log_type)}</td>
                                        )}
                                        <td style={{ padding: '6px 14px' }}>{levelBadge(l.level)}</td>
                                        {logType === 'action' && (
                                            <td style={{ padding: '6px 14px' }}>{sourceBadge(l.source)}</td>
                                        )}
                                        <td style={{ padding: '6px 14px', fontSize: 12, color: 'var(--text2)' }}>
                                            {l.module || l.category || '-'}
                                        </td>
                                        {logType === 'action' && (
                                            <td style={{ padding: '6px 14px', fontSize: 12, fontWeight: 500 }}>
                                                {l.admin_user || '-'}
                                            </td>
                                        )}
                                        <td style={{ padding: '6px 14px', fontSize: 13 }}>
                                            <div>{l.message}</div>
                                            {l.detail && (
                                                <div style={{
                                                    fontSize: 11, color: 'var(--text3)', marginTop: 3,
                                                    maxHeight: 60, overflow: 'hidden',
                                                    whiteSpace: 'pre-wrap', wordBreak: 'break-all',
                                                    fontFamily: l.detail.length > 50 ? 'monospace' : 'inherit',
                                                    background: l.detail.length > 50 ? 'var(--bg3)' : 'transparent',
                                                    padding: l.detail.length > 50 ? '4px 6px' : 0,
                                                    borderRadius: 4,
                                                }}>
                                                    {l.detail}
                                                </div>
                                            )}
                                        </td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    )}
                    <Pagination page={page} total={total} perPage={50} onPageChange={setPage} />
                </div>
            </div>
        </div>
    );
}
