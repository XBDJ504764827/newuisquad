'use client';

import { useState, useEffect, useRef, useCallback } from 'react';
import { useServers } from '../../lib/useServers';
import { api } from '../../lib/api';

interface LogEntry { log_level: string; category: string | null; message: string; raw_line: string | null; logged_at: string; }
interface ChatMsg { time: Date; player: string; message: string; channel: string; }
interface BanEntry { player_name: string; steam_id: string; duration: string; reason: string; admin: string; }
interface WarnEntry { player_name: string; steam_id: string; reason: string; admin: string; }
interface ServerInfo { server_name: string; player_count: number; max_players: number; map_name: string; game_mode: string; next_map: string; next_layer: string; }

const QUICK_COMMANDS = [
    { label: '列出玩家', cmd: 'ListPlayers' },
    { label: '列出小队', cmd: 'ListSquads' },
    { label: '下张地图', cmd: 'ShowNextMap' },
    { label: '服务器信息', cmd: 'ShowServerInfo' },
    { label: '换图确认', cmd: 'AdminSlomo 1' },
];

export default function ControlPanelPage() {
    const { servers } = useServers();
    const [selectedServer, setSelectedServer] = useState<any>(null);
    const [showAddModal, setShowAddModal] = useState(false);
    const [logs, setLogs] = useState<LogEntry[]>([]);
    const [chatMsgs, setChatMsgs] = useState<ChatMsg[]>([]);
    const [notifications, setNotifications] = useState<Array<{id: number; text: string; type: string}>>([]);
    const [rconCommand, setRconCommand] = useState('');
    const [rconResult, setRconResult] = useState('');
    const [loading, setLoading] = useState(true);
    const wsRef = useRef<WebSocket | null>(null);
    const logsEndRef = useRef<HTMLDivElement>(null);
    const chatEndRef = useRef<HTMLDivElement>(null);
    const notifId = useRef(0);

    const [form, setForm] = useState({ name: '', ip: '', rcon_port: 28016, rcon_password: '', admin_user: 'Admin' });
    const [submitting, setSubmitting] = useState(false);
    const [newToken, setNewToken] = useState('');
    const [error, setError] = useState('');
    const [deleting, setDeleting] = useState(false);
    const [deleteTarget, setDeleteTarget] = useState<any>(null);
    const [deleteError, setDeleteError] = useState('');
    const [serverState, setServerState] = useState<any>(null);
    const [serverStateLoading, setServerStateLoading] = useState(false);
    const [actionMsg, setActionMsg] = useState('');
    const [selectedTeam, setSelectedTeam] = useState(1);
    const [activeTab, setActiveTab] = useState<'players' | 'bans' | 'warns'>('players');
    const [autoRefresh, setAutoRefresh] = useState(true);
    const [bans, setBans] = useState<BanEntry[]>([]);
    const [warns, setWarns] = useState<WarnEntry[]>([]);
    const [serverInfo, setServerInfo] = useState<ServerInfo | null>(null);
    const [broadcastMsg, setBroadcastMsg] = useState('');
    const [rightTab, setRightTab] = useState<'control' | 'chat' | 'logs'>('control');

    useEffect(() => {
        if (servers.length > 0 && !selectedServer) setSelectedServer(servers[0]);
        setLoading(false);
    }, [servers, selectedServer]);

    // WebSocket 实时日志 + 聊天解析
    useEffect(() => {
        if (!selectedServer) return;
        if (wsRef.current) wsRef.current.close();
        const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const token = localStorage.getItem('token') || '';
        const ws = new WebSocket(`${proto}//${window.location.host}/api/v1/servers/${selectedServer.id}/logs/stream?token=${encodeURIComponent(token)}`);
        ws.onmessage = (e) => {
            try {
                const entry = JSON.parse(e.data);
                if (!entry.message || !entry.log_level) return;
                setLogs(prev => [...prev.slice(-300), entry]);

                // 解析聊天消息
                const cat = entry.category || '';
                if (cat.startsWith('Chat-')) {
                    const channel = cat.replace('Chat-', '');
                    const colon = entry.message.indexOf(': ');
                    const player = colon > 0 ? entry.message.slice(0, colon) : '';
                    const msg = colon > 0 ? entry.message.slice(colon + 2) : entry.message;
                    setChatMsgs(prev => [...prev.slice(-200), { time: new Date(entry.logged_at), player, message: msg, channel }]);
                }

                // 玩家进出通知
                if (cat === 'PlayerJoin') {
                    const id = ++notifId.current;
                    setNotifications(prev => [...prev.slice(-5), { id, text: entry.message, type: 'join' }]);
                    setTimeout(() => setNotifications(prev => prev.filter(n => n.id !== id)), 5000);
                }
                if (cat === 'PlayerLeave') {
                    const id = ++notifId.current;
                    setNotifications(prev => [...prev.slice(-5), { id, text: entry.message, type: 'leave' }]);
                    setTimeout(() => setNotifications(prev => prev.filter(n => n.id !== id)), 5000);
                }
            } catch { }
        };
        wsRef.current = ws;
        setLogs([]);
        setChatMsgs([]);
        return () => ws.close();
    }, [selectedServer?.id]);

    useEffect(() => { logsEndRef.current?.scrollIntoView({ behavior: 'smooth' }); }, [logs]);
    useEffect(() => { chatEndRef.current?.scrollIntoView({ behavior: 'smooth' }); }, [chatMsgs]);

    const fetchServerState = useCallback(async () => {
        if (!selectedServer) return;
        setServerStateLoading(true);
        try {
            const res = await api(`/servers/${selectedServer.id}/server-state`);
            console.log('[fetchServerState] HTTP状态:', res.status);
            const text = await res.text();
            console.log('[fetchServerState] 原始响应:', text.substring(0, 500));
            const data = JSON.parse(text);
            console.log('[fetchServerState] 解析结果:', { hasError: !!data.error, players: data.players?.length, squads: data.squads?.length, teams: data.teams });
            if (!data.error) {
                setServerState(data);
                setServerInfo(prev => ({
                    ...prev,
                    server_name: data.server_name || prev?.server_name || '',
                    player_count: data.player_count ?? prev?.player_count ?? 0,
                    max_players: data.max_players ?? prev?.max_players ?? 0,
                    map_name: data.map_name || prev?.map_name || '',
                    game_mode: data.game_mode || prev?.game_mode || '',
                    next_map: data.next_map || prev?.next_map || '',
                    next_layer: '',
                }));
            } else {
                console.warn('[fetchServerState] 服务端错误:', data.error);
            }
        } catch (e) { console.error('[fetchServerState] 请求失败', e); }
        setServerStateLoading(false);
    }, [selectedServer]);

    // 调试：监控 serverState 变化
    useEffect(() => {
        console.log('[ControlPanel] serverState 更新:', {
            hasState: !!serverState,
            players: (serverState as any)?.players?.length,
            squads: (serverState as any)?.squads?.length,
            teams: (serverState as any)?.teams,
            selectedServer: selectedServer?.id,
        });
    }, [serverState, selectedServer]);

    const fetchBansWarns = useCallback(async () => {
        if (!selectedServer) return;
        try {
            const [bRes, wRes, iRes] = await Promise.all([
                api(`/servers/${selectedServer.id}/bans`).then(r => r.json()), api(`/servers/${selectedServer.id}/warns`).then(r => r.json()), api(`/servers/${selectedServer.id}/server-info`).then(r => r.json()),
            ]);
            setBans(bRes.data || []); setWarns(wRes.data || []);
            if (!iRes.error) {
                setServerInfo(prev => ({
                    ...prev,
                    server_name: iRes.server_name || prev?.server_name || '',
                    player_count: iRes.player_count ?? prev?.player_count ?? 0,
                    max_players: iRes.max_players ?? prev?.max_players ?? 0,
                    map_name: iRes.map_name || prev?.map_name || '',
                    game_mode: iRes.game_mode || prev?.game_mode || '',
                    next_map: iRes.next_map || prev?.next_map || '',
                    next_layer: iRes.next_layer || prev?.next_layer || '',
                }));
            }
        } catch { }
    }, [selectedServer]);

    // 切换服务器时立即加载
    useEffect(() => {
        if (!selectedServer) return;
        fetchServerState(); fetchBansWarns();
    }, [selectedServer?.id]); // eslint-disable-line

    // 定时轮询
    useEffect(() => {
        if (!autoRefresh || !selectedServer) return;
        const timer = setInterval(() => { fetchServerState(); fetchBansWarns(); }, 3000);
        return () => clearInterval(timer);
    }, [autoRefresh, selectedServer, fetchServerState, fetchBansWarns]);

    const sendRcon = useCallback(async (cmd?: string) => {
        const command = cmd || rconCommand;
        if (!selectedServer || !command) return;
        const res = await api(`/servers/${selectedServer.id}/rcon`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ command, admin_user: 'Admin' }) });
        const data = await res.json();
        setRconResult(data.response || data.error || 'OK');
        if (!cmd) setRconCommand('');
    }, [selectedServer, rconCommand]);

    const execPlayerAction = useCallback(async (playerName: string, action: string, msg?: string) => {
        if (!selectedServer) return;
        try {
            const res = await api(`/servers/${selectedServer.id}/player-action`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ player_name: playerName, action, message: msg || '', admin_user: 'Admin' }) });
            const data = await res.json();
            setActionMsg(data.error ? `失败: ${data.error}` : `${action} ${playerName} 成功`);
            setTimeout(() => setActionMsg(''), 3000);
            fetchServerState();
        } catch { setActionMsg('请求失败'); }
    }, [selectedServer, fetchServerState]);

    const execDisbandSquad = useCallback(async (teamId: number, squadId: string) => {
        if (!selectedServer) return;
        try { await api(`/servers/${selectedServer.id}/disband-squad/${teamId}/${squadId}`, { method: 'DELETE' }); setActionMsg('小队已解散'); setTimeout(() => setActionMsg(''), 3000); fetchServerState(); } catch { setActionMsg('解散失败'); }
    }, [selectedServer, fetchServerState]);

    const sendBroadcast = useCallback(async () => {
        if (!selectedServer || !broadcastMsg) return;
        await api(`/servers/${selectedServer.id}/rcon`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ command: `AdminBroadcast "${broadcastMsg}"`, admin_user: 'Admin' }) });
        setActionMsg('广播已发送'); setTimeout(() => setActionMsg(''), 3000); setBroadcastMsg('');
    }, [selectedServer, broadcastMsg]);

    const handleAddServer = useCallback(async () => {
        setSubmitting(true); setError('');
        const data = await api('/servers', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ ...form, rcon_port: form.rcon_port || 28016 }) }).then(r => r.json());
        setSubmitting(false);
        if (data.error) { setError(data.error); } else { setNewToken(data.token); setSelectedServer(data); }
    }, [form]);

    const handleDeleteClick = useCallback((s: any) => { setDeleteTarget(s); setDeleteError(''); }, []);
    const handleConfirmDelete = useCallback(async () => {
        if (!deleteTarget) return; setDeleting(true);
        try {
            const res = await api(`/servers/${deleteTarget.id}`, { method: 'DELETE' });
            if (res.ok) { if (selectedServer?.id === deleteTarget.id) setSelectedServer(null); setDeleteTarget(null); }
            else { const d = await res.json(); setDeleteError(d.error || '删除失败'); }
        } catch { setDeleteError('请求失败'); }
        setDeleting(false);
    }, [deleteTarget, selectedServer]);

    if (loading) return <div className="page-view"><div className="card"><div className="card-body" style={{ textAlign: 'center', color: 'var(--text3)', padding: 40 }}>加载中...</div></div></div>;
    if (servers.length === 0 && !showAddModal) return <div className="page-view"><div className="card"><div className="empty-state"><h3>暂无服务器</h3><button className="rcon-btn" style={{ marginTop: 20 }} onClick={() => { setShowAddModal(true); setNewToken(''); setError(''); }}>添加服务器</button></div></div></div>;

    return (
        <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
            {/* 顶部栏 */}
            <div style={{ display: 'flex', gap: 8, alignItems: 'center', flexWrap: 'wrap', justifyContent: 'space-between' }}>
                <div style={{ display: 'flex', gap: 4, alignItems: 'center', flexWrap: 'wrap' }}>
                    {servers.map((s: any) => (
                        <button key={s.id} className={`tab-btn ${selectedServer?.id === s.id ? 'active' : ''}`} style={{ borderBottom: selectedServer?.id === s.id ? '2px solid var(--text)' : '2px solid transparent' }} onClick={() => setSelectedServer(s)}>{s.name}</button>
                    ))}
                    <button className="icon-btn" onClick={() => { setShowAddModal(true); setNewToken(''); setError(''); }}>+</button>
                </div>
                <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
                    <label style={{ fontSize: 11, color: 'var(--text3)', display: 'flex', gap: 4, cursor: 'pointer' }}>
                        <input type="checkbox" checked={autoRefresh} onChange={e => setAutoRefresh(e.target.checked)} />自动刷新
                    </label>
                    <button className="rcon-btn" style={{ padding: '4px 10px', fontSize: 11, width: 'auto' }} onClick={() => { fetchServerState(); fetchBansWarns(); }}>🔄</button>
                </div>
            </div>

            {/* 通知 */}
            {notifications.map(n => (
                <div key={n.id} style={{ padding: '6px 12px', fontSize: 12, borderRadius: 6, background: n.type === 'join' ? 'rgba(34,197,94,0.12)' : 'rgba(239,68,68,0.1)', color: n.type === 'join' ? '#22c55e' : 'var(--red)', animation: 'fadeIn 0.3s' }}>{n.text}</div>
            ))}

            {/* 服务器信息条 */}
            {serverInfo && (
                <div className="card" style={{ background: 'var(--bg3)' }}>
                    <div className="card-body" style={{ padding: '8px 16px', display: 'flex', gap: 16, flexWrap: 'wrap', alignItems: 'center', fontSize: 12 }}>
                        <span><strong>{serverInfo.server_name || selectedServer?.name}</strong></span>
                        <span style={{ color: '#22c55e' }}>👥 {serverInfo.player_count}/{serverInfo.max_players}</span>
                        <span>🗺️ {serverInfo.map_name} ({serverInfo.game_mode})</span>
                        <span style={{ color: 'var(--text2)' }}>→ {serverInfo.next_map}</span>
                    </div>
                </div>
            )}
            {actionMsg && <div style={{ padding: '8px 14px', fontSize: 12, borderRadius: 'var(--radius)', background: actionMsg.includes('失败') ? 'rgba(239,68,68,0.1)' : 'rgba(34,197,94,0.1)', color: actionMsg.includes('失败') ? 'var(--red)' : '#22c55e' }}>{actionMsg}</div>}

            <div style={{ display: 'grid', gridTemplateColumns: '320px 1fr', gap: 20, alignItems: 'start' }}>
                {/* 左侧面板 */}
                <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
                    {/* 服务器信息 */}
                    {selectedServer && (
                        <div className="card">
                            <div className="card-header"><div className="card-title">服务器信息</div></div>
                            <div className="card-body" style={{ display: 'flex', flexDirection: 'column', gap: 10, fontSize: 13 }}>
                                <InfoRow label="ID" value={selectedServer.server_id} />
                                <InfoRow label="IP" value={`${selectedServer.ip}:${selectedServer.rcon_port}`} />
                                <button onClick={() => handleDeleteClick(selectedServer)} style={{ width: '100%', padding: '6px 0', background: 'transparent', border: '1px solid var(--red)', borderRadius: 'var(--radius)', color: 'var(--red)', cursor: 'pointer', fontSize: 12 }}>删除服务器</button>
                            </div>
                        </div>
                    )}

                    {/* 快捷命令 */}
                    {selectedServer && (
                        <div className="card">
                            <div className="card-header"><div className="card-title">快捷命令</div></div>
                            <div className="card-body" style={{ display: 'flex', flexWrap: 'wrap', gap: 4 }}>
                                {QUICK_COMMANDS.map(qc => (
                                    <button key={qc.cmd} className="rcon-btn" style={{ fontSize: 11, padding: '4px 10px', width: 'auto' }} onClick={() => sendRcon(qc.cmd)}>{qc.label}</button>
                                ))}
                            </div>
                        </div>
                    )}

                    {/* RCON 命令 */}
                    {selectedServer && (
                        <div className="card">
                            <div className="card-header"><div className="card-title">RCON</div></div>
                            <div className="card-body">
                                <input type="text" className="rcon-input" placeholder="输入指令..." value={rconCommand} onChange={e => setRconCommand(e.target.value)} onKeyDown={e => e.key === 'Enter' && sendRcon()} />
                                <button className="rcon-btn" style={{ marginTop: 6 }} onClick={() => sendRcon()}>发送</button>
                                {rconResult && <div className="terminal" style={{ marginTop: 8, maxHeight: 120, overflowY: 'auto', fontSize: 11, padding: 8, whiteSpace: 'pre-wrap' }}>{rconResult}</div>}
                            </div>
                        </div>
                    )}

                    {/* 快速广播 */}
                    {selectedServer && (
                        <div className="card">
                            <div className="card-header"><div className="card-title">广播</div></div>
                            <div className="card-body">
                                <input type="text" className="rcon-input" placeholder="广播内容..." value={broadcastMsg} onChange={e => setBroadcastMsg(e.target.value)} onKeyDown={e => e.key === 'Enter' && sendBroadcast()} />
                                <button className="rcon-btn" style={{ marginTop: 6 }} onClick={sendBroadcast}>发送</button>
                            </div>
                        </div>
                    )}
                </div>

                {/* 右侧面板 */}
                <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
                    {/* 右侧标签：控制 / 聊天 / 日志 */}
                    <div className="card">
                        <div style={{ display: 'flex', borderBottom: '1px solid var(--border)' }}>
                            {(['control', 'chat', 'logs'] as const).map(tab => (
                                <button key={tab} onClick={() => setRightTab(tab)} style={{ flex: 1, padding: '10px', fontSize: 12, border: 'none', background: 'transparent', cursor: 'pointer', borderBottom: rightTab === tab ? '2px solid var(--text)' : '2px solid transparent', color: rightTab === tab ? 'var(--text)' : 'var(--text3)', fontWeight: rightTab === tab ? 600 : 400 }}>
                                    {tab === 'control' ? '👥 玩家控制' : tab === 'chat' ? `💬 实时聊天 (${chatMsgs.length})` : `📋 日志 (${logs.length})`}
                                </button>
                            ))}
                        </div>

                        {/* 玩家控制 */}
                        {rightTab === 'control' && selectedServer && (
                            <div>
                                {/* 子标签 */}
                                <div style={{ display: 'flex', borderBottom: '1px solid var(--border)' }}>
                                    {(['players', 'bans', 'warns'] as const).map(tab => (
                                        <button key={tab} onClick={() => setActiveTab(tab)} style={{ padding: '6px 14px', fontSize: 11, border: 'none', background: 'transparent', cursor: 'pointer', borderBottom: activeTab === tab ? '2px solid var(--text)' : '2px solid transparent', color: activeTab === tab ? 'var(--text)' : 'var(--text3)' }}>
                                            {tab === 'players' ? '玩家' : tab === 'bans' ? '封禁' : '警告'}
                                        </button>
                                    ))}
                                </div>
                                <div className="card-body" style={{ padding: 0 }}>
                                    {activeTab === 'players' && (serverState ? (
                                        <div>
                                            <div style={{ display: 'flex', borderBottom: '1px solid var(--border)' }}>
                                                {[
                                                    ...(serverState.teams && serverState.teams.length > 0
                                                        ? serverState.teams
                                                        : [{team_id: 1, faction: '队伍 1'}, {team_id: 2, faction: '队伍 2'}]),
                                                    {team_id: 0, faction: '观战/部署'},
                                                ].map((t: any) => {
                                                    const cnt = (serverState.players || []).filter((p: any) => p.team_id === t.team_id).length;
                                                    if (t.team_id === 0 && cnt === 0) return null;
                                                    return (
                                                        <div key={t.team_id} onClick={() => setSelectedTeam(t.team_id)} style={{ flex: 1, padding: '8px', cursor: 'pointer', textAlign: 'center', fontSize: 12, borderBottom: selectedTeam === t.team_id ? '2px solid var(--text)' : '2px solid transparent', color: selectedTeam === t.team_id ? 'var(--text)' : 'var(--text3)' }}>{t.faction} ({cnt})</div>
                                                    );
                                                })}
                                            </div>
                                            {[selectedTeam].map(teamId => {
                                                const tp = (serverState.players || []).filter((p: any) => p.team_id === teamId);
                                                const ts = (serverState.squads || []).filter((s: any) => s.team_id === teamId);
                                                const sp = (sid: string | null) => tp.filter((p: any) => p.squad_id === sid);
                                                const us = tp.filter((p: any) => !p.squad_id);
                                                // 找出有 squad_id 但 squad 列表里不存在的孤立玩家
                                                const orphanSquadIds = new Set<string>();
                                                tp.forEach((p: any) => {
                                                    if (p.squad_id && !ts.some((s: any) => s.squad_id === p.squad_id)) {
                                                        orphanSquadIds.add(p.squad_id);
                                                    }
                                                });
                                                return (
                                                    <div key={teamId} style={{ maxHeight: 400, overflowY: 'auto' }}>
                                                        {ts.map((sq: any) => (
                                                            <div key={sq.name} style={{ borderBottom: '1px solid var(--border)' }}>
                                                                <div style={{ display: 'flex', justifyContent: 'space-between', padding: '6px 12px', background: 'var(--bg3)', fontSize: 12 }}>
                                                                    <strong>{sq.name}</strong>
                                                                    <span style={{ color: 'var(--text3)', fontSize: 11 }}>{sq.creator} ({sp(sq.squad_id).length})</span>
                                                                    <span className="badge red" style={{ cursor: 'pointer', fontSize: 9 }} onClick={() => { if (confirm(`解散 ${sq.name}?`)) execDisbandSquad(teamId, sq.squad_id); }}>解散</span>
                                                                </div>
                                                                <PlayerTable players={sp(sq.squad_id)} onAction={execPlayerAction} />
                                                            </div>
                                                        ))}
                                                        {Array.from(orphanSquadIds).map(sid => (
                                                            <div key={`orphan-${sid}`} style={{ borderBottom: '1px solid var(--border)' }}>
                                                                <div style={{ display: 'flex', justifyContent: 'space-between', padding: '6px 12px', background: 'var(--bg3)', fontSize: 12 }}>
                                                                    <strong>小队 {sid}</strong>
                                                                    <span style={{ color: 'var(--text3)', fontSize: 11 }}>({sp(sid).length})</span>
                                                                </div>
                                                                <PlayerTable players={sp(sid)} onAction={execPlayerAction} />
                                                            </div>
                                                        ))}
                                                        {us.length > 0 && <div style={{ borderBottom: '1px solid var(--border)' }}><div style={{ padding: '6px 12px', background: 'var(--bg3)', fontSize: 12, color: 'var(--text3)' }}>未入队 ({us.length})</div><PlayerTable players={us} onAction={execPlayerAction} /></div>}
                                                    </div>
                                                );
                                            })}
                                        </div>
                                    ) : <div style={{ padding: 30, textAlign: 'center', color: 'var(--text3)' }}>
                                        <p>点击刷新获取玩家列表</p>
                                        <p style={{ fontSize: 10, marginTop: 8 }}>serverState: {JSON.stringify(serverState)}</p>
                                        <p style={{ fontSize: 10 }}>selectedServer: {selectedServer?.id}</p>
                                        <p style={{ fontSize: 10 }}>rightTab: {rightTab} | activeTab: {activeTab}</p>
                                    </div>)}
                                    {activeTab === 'bans' && (bans.length > 0 ? <table style={{ width: '100%', fontSize: 12 }}><thead><tr style={{ borderBottom: '2px solid var(--border)' }}><th style={{ padding: '8px 12px', color: 'var(--text3)' }}>玩家</th><th style={{ padding: '8px 12px', color: 'var(--text3)' }}>时长</th><th style={{ padding: '8px 12px', color: 'var(--text3)' }}>原因</th></tr></thead><tbody>{bans.map((b, i) => <tr key={i} style={{ borderBottom: '1px solid var(--border)' }}><td style={{ padding: '6px 12px' }}>{b.player_name}</td><td style={{ padding: '6px 12px' }}><span className="badge red">{b.duration}</span></td><td style={{ padding: '6px 12px', color: 'var(--text2)' }}>{b.reason}</td></tr>)}</tbody></table> : <div style={{ padding: 20, textAlign: 'center', color: 'var(--text3)' }}>✅ 无封禁</div>)}
                                    {activeTab === 'warns' && (warns.length > 0 ? <table style={{ width: '100%', fontSize: 12 }}><thead><tr style={{ borderBottom: '2px solid var(--border)' }}><th style={{ padding: '8px 12px', color: 'var(--text3)' }}>玩家</th><th style={{ padding: '8px 12px', color: 'var(--text3)' }}>原因</th></tr></thead><tbody>{warns.map((w, i) => <tr key={i} style={{ borderBottom: '1px solid var(--border)' }}><td style={{ padding: '6px 12px' }}>{w.player_name}</td><td style={{ padding: '6px 12px', color: 'var(--text2)' }}>{w.reason}</td></tr>)}</tbody></table> : <div style={{ padding: 20, textAlign: 'center', color: 'var(--text3)' }}>✅ 无警告</div>)}
                                </div>
                            </div>
                        )}

                        {/* 实时聊天 */}
                        {rightTab === 'chat' && (
                            <div className="terminal" style={{ border: 'none', borderRadius: 0, height: 500, overflowY: 'auto', padding: '10px 14px', fontFamily: 'monospace', fontSize: 12, display: 'flex', flexDirection: 'column', gap: 4 }}>
                                {chatMsgs.length === 0 && <div style={{ color: 'var(--text3)', textAlign: 'center', padding: 40 }}>等待聊天消息...</div>}
                                {chatMsgs.map((c, i) => (
                                    <div key={i} style={{ lineHeight: 1.5 }}>
                                        <span style={{ color: 'var(--text3)', fontSize: 10 }}>{c.time.toLocaleTimeString()} </span>
                                        <span style={{ color: c.channel === 'Team' ? '#3b82f6' : c.channel === 'Squad' ? '#22c55e' : c.channel === 'Admin' ? '#f59e0b' : '#a78bfa', fontSize: 10 }}>[{c.channel}] </span>
                                        <span style={{ fontWeight: 600 }}>{c.player}</span>
                                        <span style={{ color: 'var(--text2)' }}>: {c.message}</span>
                                    </div>
                                ))}
                                <div ref={chatEndRef} />
                            </div>
                        )}

                        {/* 实时日志 */}
                        {rightTab === 'logs' && (
                            <div className="terminal" style={{ border: 'none', borderRadius: 0, height: 500, overflowY: 'auto', padding: '10px 14px', fontFamily: 'monospace', fontSize: 11 }}>
                                {logs.length === 0 && <div style={{ color: 'var(--text3)' }}>等待日志...</div>}
                                {logs.map((entry, i) => (
                                    <div key={i}>
                                        <span className="time">[{new Date(entry.logged_at).toLocaleTimeString()}]</span>
                                        <span className={entry.log_level === 'ERROR' ? 'error' : entry.log_level === 'WARN' ? 'warn' : 'info'}>[{entry.category || 'General'}]</span> {entry.message}
                                    </div>
                                ))}
                                <div ref={logsEndRef}>_</div>
                            </div>
                        )}
                    </div>
                </div>
            </div>

            {/* 删除确认 Modal */}
            {deleteTarget && (
                <Modal onClose={() => setDeleteTarget(null)}>
                    <h3 style={{ color: 'var(--red)', marginBottom: 12 }}>删除服务器</h3>
                    <p style={{ color: 'var(--text2)', fontSize: 13, marginBottom: 16 }}>确定删除 {deleteTarget.name}？此操作不可撤销。</p>
                    {deleteError && <div style={{ color: 'var(--red)', marginBottom: 12, fontSize: 12 }}>{deleteError}</div>}
                    <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
                        <button className="rcon-btn" style={{ background: 'var(--bg3)' }} onClick={() => setDeleteTarget(null)} disabled={deleting}>取消</button>
                        <button onClick={handleConfirmDelete} disabled={deleting} style={{ padding: '8px 20px', background: 'var(--red)', border: 'none', borderRadius: 'var(--radius)', color: '#fff', cursor: 'pointer', fontSize: 13, opacity: deleting ? 0.5 : 1 }}>{deleting ? '删除中...' : '确认删除'}</button>
                    </div>
                </Modal>
            )}

            {/* 添加服务器 Modal */}
            {showAddModal && (
                <Modal onClose={() => setShowAddModal(false)}>
                    <h3 style={{ marginBottom: 16 }}>添加游戏服务器</h3>
                    {newToken ? (
                        <div>
                            <div className="terminal" style={{ padding: 12, marginBottom: 12, wordBreak: 'break-all', fontSize: 13 }}>
                                Agent Token：<br />
                                <code style={{ color: '#22c55e' }}>{newToken}</code>
                            </div>
                            <button className="rcon-btn" onClick={() => setShowAddModal(false)}>关闭</button>
                        </div>
                    ) : (
                        <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
                            <input className="rcon-input" placeholder="服务器名称" value={form.name} onChange={e => setForm({ ...form, name: e.target.value })} />
                            <input className="rcon-input" placeholder="服务器 IP" value={form.ip} onChange={e => setForm({ ...form, ip: e.target.value })} />
                            <input className="rcon-input" type="number" placeholder="RCON 端口" value={form.rcon_port || ''} onChange={e => setForm({ ...form, rcon_port: parseInt(e.target.value) || 0 })} />
                            <input className="rcon-input" type="password" placeholder="RCON 密码" value={form.rcon_password} onChange={e => setForm({ ...form, rcon_password: e.target.value })} />
                            {error && <div style={{ color: 'var(--red)', fontSize: 12 }}>{error}</div>}
                            <div style={{ display: 'flex', gap: 8, marginTop: 4 }}>
                                <button className="rcon-btn" onClick={handleAddServer} disabled={submitting}>{submitting ? '验证中...' : '验证并添加'}</button>
                                <button className="rcon-btn" style={{ background: 'var(--bg3)' }} onClick={() => setShowAddModal(false)}>取消</button>
                            </div>
                        </div>
                    )}
                </Modal>
            )}
        </div>
    );
}

function InfoRow({ label, value }: { label: string; value: string }) {
    return <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid var(--border)', paddingBottom: 6 }}><span style={{ color: 'var(--text3)', fontSize: 11 }}>{label}</span><span style={{ fontWeight: 600 }}>{value}</span></div>;
}

function PlayerTable({ players, onAction }: { players: any[]; onAction: (name: string, action: string, msg?: string) => void }) {
    return (
        <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 11 }}>
            <thead><tr style={{ borderBottom: '1px solid var(--border)' }}><th style={{ padding: '4px 6px', color: 'var(--text3)', fontWeight: 500, textAlign: 'left' }}>玩家</th><th style={{ padding: '4px 6px', color: 'var(--text3)', fontWeight: 500, textAlign: 'left' }}>职业</th><th style={{ padding: '4px 6px', color: 'var(--text3)', textAlign: 'center' }}>K</th><th style={{ padding: '4px 6px', color: 'var(--text3)', textAlign: 'center' }}>D</th><th style={{ padding: '4px 6px', color: 'var(--text3)', textAlign: 'left' }}>操作</th></tr></thead>
            <tbody>{players.map((p: any) => (
                <tr key={p.name + (p.steam_id || '')} style={{ borderBottom: '1px solid var(--border)' }}>
                    <td style={{ padding: '4px 6px', fontWeight: 500 }}>{p.name}{p.is_admin && <span style={{ color: '#f59e0b', fontSize: 9, marginLeft: 4 }}>ADMIN</span>}</td>
                    <td style={{ padding: '4px 6px', fontSize: 10, color: 'var(--text2)' }}>{p.role}</td>
                    <td style={{ padding: '4px 6px', textAlign: 'center', color: '#22c55e' }}>{p.kills}</td>
                    <td style={{ padding: '4px 6px', textAlign: 'center', color: 'var(--red)' }}>{p.deaths}</td>
                    <td style={{ padding: '4px 6px' }}><div style={{ display: 'flex', gap: 2, flexWrap: 'wrap' }}>
                        <span className="badge gray" style={{ cursor: 'pointer', fontSize: 9 }} onClick={() => onAction(p.name, 'warn')}>警告</span>
                        <span className="badge red" style={{ cursor: 'pointer', fontSize: 9 }} onClick={() => { if (confirm(`踢出 ${p.name}?`)) onAction(p.name, 'kick', '管理员操作'); }}>踢出</span>
                        <span className="badge red" style={{ cursor: 'pointer', fontSize: 9 }} onClick={() => { if (confirm(`封禁 ${p.name}?`)) onAction(p.name, 'ban', '管理员操作'); }}>封禁</span>
                        <span className="badge" style={{ cursor: 'pointer', fontSize: 9, background: 'var(--blue)', color: '#fff' }} onClick={() => { if (confirm(`强制 ${p.name} 跳边?`)) onAction(p.name, 'team_change'); }}>跳边</span>
                        <span className="badge" style={{ cursor: 'pointer', fontSize: 9, background: '#f59e0b', color: '#000' }} onClick={() => { if (confirm(`将 ${p.name} 踢出小队?`)) onAction(p.name, 'squad_remove'); }}>踢出小队</span>
                    </div></td>
                </tr>
            ))}</tbody>
        </table>
    );
}

function Modal({ children, onClose }: { children: React.ReactNode; onClose: () => void }) {
    return <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.6)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 1000 }} onClick={e => { if (e.target === e.currentTarget) onClose(); }}><div style={{ background: 'var(--bg2)', border: '1px solid var(--border)', borderRadius: 'var(--radius)', padding: 24, width: 440, maxWidth: '90vw' }} onClick={e => e.stopPropagation()}>{children}</div></div>;
}
