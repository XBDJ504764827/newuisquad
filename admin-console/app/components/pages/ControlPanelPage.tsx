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
    { label: '列出玩家', cmd: 'ListPlayers', icon: '👥' },
    { label: '列出小队', cmd: 'ListSquads', icon: '🛡️' },
    { label: '下张地图', cmd: 'ShowNextMap', icon: '🗺️' },
    { label: '服务器信息', cmd: 'ShowServerInfo', icon: '📊' },
    { label: '结束对局', cmd: 'AdminEndMatch', icon: '🏁' },
    { label: '换图确认', cmd: 'AdminSlomo 1', icon: '⏱️' },
];

const CHANNEL_COLORS: Record<string, string> = {
    All: '#a78bfa', Team: '#3b82f6', Squad: '#22c55e', Admin: '#f59e0b',
};

const LOG_LEVEL_COLORS: Record<string, string> = {
    ERROR: '#ef4444', WARN: '#f59e0b', INFO: '#3b82f6', SUCCESS: '#22c55e', DEBUG: '#71717a',
};

function factionFlag(f: string) {
    if (/pla|people.*liberation/i.test(f)) return '🇨🇳';
    if (/us\s*army|united\s*states/i.test(f)) return '🇺🇸';
    if (/british|baf/i.test(f)) return '🇬🇧';
    if (/canadian/i.test(f)) return '🇨🇦';
    if (/australian/i.test(f)) return '🇦🇺';
    if (/russian|rgf|vdv/i.test(f)) return '🇷🇺';
    if (/insurgent|irregular/i.test(f)) return '🏴';
    if (/turkish/i.test(f)) return '🇹🇷';
    if (/middle\s*eastern|mea/i.test(f)) return '🇸🇦';
    if (/marine/i.test(f)) return '🌎';
    return '🎖️';
}

export default function ControlPanelPage() {
    const { servers } = useServers();
    const [selectedServer, setSelectedServer] = useState<any>(null);
    const [showAddModal, setShowAddModal] = useState(false);
    const [logs, setLogs] = useState<LogEntry[]>([]);
    const [chatMsgs, setChatMsgs] = useState<ChatMsg[]>([]);
    const [notifications, setNotifications] = useState<Array<{ id: number; text: string; type: string }>>([]);
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
    // 暖服作弊开关状态: null=未知, true=开启, false=关闭
    const [warmupToggles, setWarmupToggles] = useState<Record<string, boolean | null>>(() => {
        try { const v = localStorage.getItem('warmupToggles'); if (v) return JSON.parse(v); } catch {}
        return {};
    });
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
                const cat = entry.category || '';
                if (cat.startsWith('Chat-')) {
                    const channel = cat.replace('Chat-', '');
                    const colon = entry.message.indexOf(': ');
                    const player = colon > 0 ? entry.message.slice(0, colon) : '';
                    const msg = colon > 0 ? entry.message.slice(colon + 2) : entry.message;
                    setChatMsgs(prev => [...prev.slice(-200), { time: new Date(entry.logged_at), player, message: msg, channel }]);
                }
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
            const data = await res.json();
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
            }
        } catch { }
        setServerStateLoading(false);
    }, [selectedServer]);

    const fetchBansWarns = useCallback(async () => {
        if (!selectedServer) return;
        try {
            const [bRes, wRes, iRes] = await Promise.all([
                api(`/servers/${selectedServer.id}/bans`).then(r => r.json()),
                api(`/servers/${selectedServer.id}/warns`).then(r => r.json()),
                api(`/servers/${selectedServer.id}/server-info`).then(r => r.json()),
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

    useEffect(() => {
        if (!selectedServer) return;
        fetchServerState(); fetchBansWarns();
    }, [selectedServer?.id]);

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

    const playerPct = serverInfo ? Math.round((serverInfo.player_count / Math.max(1, serverInfo.max_players)) * 100) : 0;

    if (loading) return <div className="page-view"><div className="card"><div className="card-body" style={{ textAlign: 'center', color: 'var(--text3)', padding: 48 }}>加载中...</div></div></div>;
    if (servers.length === 0 && !showAddModal) return <div className="page-view"><div className="card"><div className="empty-state"><h3 style={{ fontSize: 18, marginBottom: 8 }}>暂无服务器</h3><p style={{ color: 'var(--text3)', marginBottom: 20 }}>添加游戏服务器以开始管理</p><button className="rcon-btn" style={{ width: 'auto', paddingLeft: 24, paddingRight: 24 }} onClick={() => { setShowAddModal(true); setNewToken(''); setError(''); }}>+ 添加服务器</button></div></div></div>;

    return (
        <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
            {/* ═══ 顶栏：服务器选择 + 信息条 ═══ */}
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 12, flexWrap: 'wrap' }}>
                <div style={{ display: 'flex', gap: 4, alignItems: 'center' }}>
                    {servers.map((s: any) => (
                        <button
                            key={s.id}
                            onClick={() => setSelectedServer(s)}
                            style={{
                                padding: '6px 14px', borderRadius: 6, border: 'none', cursor: 'pointer', fontSize: 12, fontWeight: 500,
                                background: selectedServer?.id === s.id ? 'var(--text)' : 'var(--bg3)',
                                color: selectedServer?.id === s.id ? 'var(--bg)' : 'var(--text2)',
                                transition: 'all .15s',
                            }}
                        >{s.name}</button>
                    ))}
                    <button
                        onClick={() => { setShowAddModal(true); setNewToken(''); setError(''); }}
                        style={{ width: 28, height: 28, borderRadius: 6, border: '1px dashed var(--border2)', background: 'transparent', color: 'var(--text3)', cursor: 'pointer', fontSize: 16, display: 'flex', alignItems: 'center', justifyContent: 'center', transition: 'all .15s' }}
                        title="添加服务器"
                    >+</button>
                </div>
                <div style={{ display: 'flex', gap: 10, alignItems: 'center' }}>
                    <label style={{ fontSize: 11, color: 'var(--text3)', display: 'flex', gap: 5, alignItems: 'center', cursor: 'pointer', userSelect: 'none' }}>
                        <div style={{ width: 32, height: 18, borderRadius: 9, background: autoRefresh ? '#22c55e' : 'var(--border2)', position: 'relative', transition: 'background .2s' }}>
                            <div style={{ position: 'absolute', top: 2, left: autoRefresh ? 16 : 2, width: 14, height: 14, borderRadius: '50%', background: '#fff', transition: 'left .2s', boxShadow: '0 1px 3px rgba(0,0,0,.3)' }} />
                        </div>
                        自动刷新
                    </label>
                    <button
                        onClick={() => { fetchServerState(); fetchBansWarns(); }}
                        style={{ width: 28, height: 28, borderRadius: 6, border: '1px solid var(--border)', background: 'var(--bg3)', color: 'var(--text2)', cursor: 'pointer', fontSize: 13, display: 'flex', alignItems: 'center', justifyContent: 'center', transition: 'all .15s' }}
                        title="手动刷新"
                    >🔄</button>
                </div>
            </div>

            {/* ═══ 通知区域 ═══ */}
            {notifications.map(n => (
                <div key={n.id} className="badge" style={{
                    padding: '8px 14px', fontSize: 12, borderRadius: 6, animation: 'fadeIn .3s', fontWeight: 500,
                    background: n.type === 'join' ? 'rgba(34,197,94,0.1)' : 'rgba(239,68,68,0.1)',
                    color: n.type === 'join' ? '#22c55e' : 'var(--red)',
                    border: `1px solid ${n.type === 'join' ? 'rgba(34,197,94,0.2)' : 'rgba(239,68,68,0.2)'}`,
                }}>{n.type === 'join' ? '✅' : '👋'} {n.text}</div>
            ))}

            {/* ═══ 服务器状态卡片 ═══ */}
            {serverInfo && (
                <div style={{ background: 'var(--bg2)', border: '1px solid var(--border)', borderRadius: 10, overflow: 'hidden' }}>
                    <div style={{ padding: '14px 20px', display: 'flex', gap: 20, flexWrap: 'wrap', alignItems: 'center' }}>
                        {/* 服务器名 */}
                        <div style={{ minWidth: 0, flex: '0 0 auto' }}>
                            <div style={{ fontSize: 13, fontWeight: 600, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', maxWidth: 260 }}>
                                🖥️ {serverInfo.server_name || selectedServer?.name}
                            </div>
                        </div>

                        <div style={{ width: 1, height: 24, background: 'var(--border)', flexShrink: 0 }} />

                        {/* 玩家数 */}
                        <div style={{ display: 'flex', alignItems: 'center', gap: 10, flexShrink: 0 }}>
                            <span style={{ fontSize: 13, fontWeight: 600, whiteSpace: 'nowrap' }}>
                                👥 {serverInfo.player_count}<span style={{ color: 'var(--text3)', fontWeight: 400 }}>/{serverInfo.max_players}</span>
                            </span>
                            <div style={{ width: 80, height: 6, borderRadius: 3, background: 'var(--bg3)', overflow: 'hidden' }}>
                                <div style={{ height: '100%', borderRadius: 3, background: playerPct > 90 ? '#ef4444' : playerPct > 70 ? '#f59e0b' : '#22c55e', width: `${Math.min(100, playerPct)}%`, transition: 'width .5s ease' }} />
                            </div>
                        </div>

                        <div style={{ width: 1, height: 24, background: 'var(--border)', flexShrink: 0 }} />

                        {/* 地图信息 */}
                        <div style={{ display: 'flex', gap: 16, flexWrap: 'wrap', alignItems: 'center', flex: 1, minWidth: 0 }}>
                            <span style={{ fontSize: 13, whiteSpace: 'nowrap' }}>
                                🗺️ <strong>{serverInfo.map_name}</strong>
                                <span style={{ color: 'var(--text3)', marginLeft: 6 }}>({serverInfo.game_mode})</span>
                            </span>
                            {serverInfo.next_map && (
                                <span style={{ fontSize: 12, color: 'var(--text2)', whiteSpace: 'nowrap' }}>
                                    → <span style={{ color: 'var(--text3)' }}>下一张:</span> {serverInfo.next_map}
                                </span>
                            )}
                        </div>

                        {/* 阵营旗帜 */}
                        {serverState?.teams && serverState.teams.length >= 2 && (
                            <>
                                <div style={{ width: 1, height: 24, background: 'var(--border)', flexShrink: 0 }} />
                                <div style={{ display: 'flex', gap: 12, alignItems: 'center', flexShrink: 0, fontSize: 12 }}>
                                    <span>{factionFlag(serverState.teams[0]?.faction || '')} {serverState.teams[0]?.faction || '—'}</span>
                                    <span style={{ color: 'var(--text3)', fontWeight: 700 }}>VS</span>
                                    <span>{factionFlag(serverState.teams[1]?.faction || '')} {serverState.teams[1]?.faction || '—'}</span>
                                </div>
                            </>
                        )}
                    </div>
                </div>
            )}

            {actionMsg && (
                <div style={{ padding: '8px 14px', fontSize: 12, borderRadius: 6, background: actionMsg.includes('失败') ? 'rgba(239,68,68,0.08)' : 'rgba(34,197,94,0.08)', color: actionMsg.includes('失败') ? 'var(--red)' : '#22c55e', border: `1px solid ${actionMsg.includes('失败') ? 'rgba(239,68,68,0.15)' : 'rgba(34,197,94,0.15)'}`, fontWeight: 500 }}>
                    {actionMsg}
                </div>
            )}

            {/* ═══ 主内容区 ═══ */}
            <div style={{ display: 'grid', gridTemplateColumns: '300px 1fr', gap: 16, alignItems: 'start' }}>
                {/* ═══ 左侧面板 ═══ */}
                <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                    {/* 服务器信息卡片 */}
                    {selectedServer && (
                        <div className="card">
                            <div className="card-header" style={{ padding: '10px 14px' }}>
                                <div className="card-title" style={{ fontSize: 13 }}>📡 连接信息</div>
                            </div>
                            <div className="card-body" style={{ padding: '12px 14px', display: 'flex', flexDirection: 'column', gap: 8, fontSize: 12 }}>
                                <InfoRow label="服务器 ID" value={String(selectedServer.server_id)} />
                                <InfoRow label="地址" value={`${selectedServer.ip}:${selectedServer.rcon_port}`} />
                                <button
                                    onClick={() => handleDeleteClick(selectedServer)}
                                    style={{
                                        width: '100%', marginTop: 4, padding: '7px 0', background: 'transparent',
                                        border: '1px solid rgba(239,68,68,0.3)', borderRadius: 6,
                                        color: 'var(--red)', cursor: 'pointer', fontSize: 11, fontWeight: 500,
                                        transition: 'all .15s',
                                    }}
                                    onMouseEnter={e => { e.currentTarget.style.background = 'rgba(239,68,68,0.08)'; }}
                                    onMouseLeave={e => { e.currentTarget.style.background = 'transparent'; }}
                                >删除服务器</button>
                            </div>
                        </div>
                    )}

                    {/* RCON 命令 */}
                    {selectedServer && (
                        <div className="card">
                            <div className="card-header" style={{ padding: '10px 14px' }}>
                                <div className="card-title" style={{ fontSize: 13 }}>⌨️ RCON 命令</div>
                            </div>
                            <div className="card-body" style={{ padding: '10px 14px', display: 'flex', flexDirection: 'column', gap: 8 }}>
                                <div style={{ display: 'flex', gap: 6 }}>
                                    <input
                                        type="text"
                                        className="rcon-input"
                                        placeholder="输入 RCON 指令..."
                                        value={rconCommand}
                                        onChange={e => setRconCommand(e.target.value)}
                                        onKeyDown={e => e.key === 'Enter' && sendRcon()}
                                        style={{ flex: 1, fontSize: 12, padding: '8px 10px' }}
                                    />
                                    <button
                                        className="rcon-btn"
                                        onClick={() => sendRcon()}
                                        style={{ width: 'auto', padding: '8px 14px', fontSize: 12 }}
                                    >发送</button>
                                </div>
                                {rconResult && (
                                    <div className="terminal" style={{ maxHeight: 140, overflowY: 'auto', fontSize: 11, padding: 10, whiteSpace: 'pre-wrap', wordBreak: 'break-all', borderRadius: 6 }}>
                                        {rconResult}
                                    </div>
                                )}
                            </div>
                        </div>
                    )}

                    {/* 快捷命令 */}
                    {selectedServer && (
                        <div className="card">
                            <div className="card-header" style={{ padding: '10px 14px' }}>
                                <div className="card-title" style={{ fontSize: 13 }}>⚡ 快捷命令</div>
                            </div>
                            <div className="card-body" style={{ padding: '8px 14px', display: 'flex', flexDirection: 'column', gap: 4 }}>
                                {QUICK_COMMANDS.map(qc => (
                                    <button
                                        key={qc.cmd}
                                        onClick={() => sendRcon(qc.cmd)}
                                        style={{
                                            width: '100%', padding: '7px 12px', border: '1px solid var(--border)',
                                            borderRadius: 6, background: 'var(--bg3)', color: 'var(--text2)',
                                            cursor: 'pointer', fontSize: 12, textAlign: 'left',
                                            transition: 'all .12s', display: 'flex', gap: 8, alignItems: 'center',
                                        }}
                                        onMouseEnter={e => { e.currentTarget.style.background = 'var(--bg4)'; e.currentTarget.style.color = 'var(--text)'; }}
                                        onMouseLeave={e => { e.currentTarget.style.background = 'var(--bg3)'; e.currentTarget.style.color = 'var(--text2)'; }}
                                    ><span style={{ fontSize: 14 }}>{qc.icon}</span> {qc.label}</button>
                                ))}
                            </div>
                        </div>
                    )}

                    {/* 暖服功能 */}
                    {selectedServer && (
                        <div className="card">
                            <div className="card-header" style={{ padding: '10px 14px' }}>
                                <div className="card-title" style={{ fontSize: 13 }}>🔥 暖服功能</div>
                                <div className="card-sub">快速开关暖服作弊选项</div>
                            </div>
                            <div className="card-body" style={{ padding: '8px 14px', display: 'flex', flexDirection: 'column', gap: 6 }}>
                                {[
                                    { key: 'novehicleclaim', label: '取消载具认领权限', cmd: 'AdminDisableVehicleClaiming' },
                                    { key: 'forcevehicle', label: '始终填满所有载具刷新位置', cmd: 'AdminForceAllVehicleAvailability' },
                                    { key: 'forcedeploy', label: '取消部署要求限制', cmd: 'AdminForceAllDeployableAvailability' },
                                    { key: 'forcerole', label: '取消装具人数限制', cmd: 'AdminForceAllRoleAvailability' },
                                    { key: 'noenemylimit', label: '可以使用敌方载具', cmd: 'AdminDisableVehicleTeamRequirement' },
                                    { key: 'nokitreq', label: '取消坦克飞机载具要求', cmd: 'AdminDisableVehicleKitRequirement' },
                                    { key: 'norespawn', label: '取消复活时间', cmd: 'AdminNoRespawnTimer' },
                                ].map(item => {
                                    const state = warmupToggles[item.key]; // null=未知, true=开启, false=关闭
                                    const setState = (v: boolean) => {
                                        sendRcon(`${item.cmd} ${v ? 1 : 0}`);
                                        const next = { ...warmupToggles, [item.key]: v };
                                        setWarmupToggles(next);
                                        try { localStorage.setItem('warmupToggles', JSON.stringify(next)); } catch {}
                                    };
                                    return (
                                        <div key={item.key} style={{
                                            display: 'flex', justifyContent: 'space-between', alignItems: 'center',
                                            padding: '6px 10px', borderRadius: 6,
                                            background: state === true ? 'rgba(34,197,94,0.06)' : state === false ? 'rgba(239,68,68,0.04)' : 'var(--bg3)',
                                            border: `1px solid ${state === true ? 'rgba(34,197,94,0.2)' : state === false ? 'rgba(239,68,68,0.15)' : 'var(--border)'}`,
                                            transition: 'all .15s',
                                        }}>
                                            <span style={{ fontSize: 12, fontWeight: 500, color: state === true ? '#22c55e' : state === false ? 'var(--red)' : 'var(--text2)' }}>{item.label}</span>
                                            <div style={{ display: 'flex', gap: 4, flexShrink: 0, marginLeft: 8 }}>
                                                <button
                                                    onClick={() => setState(true)}
                                                    style={{
                                                        padding: '2px 10px', borderRadius: 4, border: 'none',
                                                        cursor: 'pointer', fontSize: 10, fontWeight: 700,
                                                        background: state === true ? '#22c55e' : 'rgba(34,197,94,0.12)',
                                                        color: state === true ? '#fff' : 'rgba(34,197,94,0.6)',
                                                        transition: 'all .1s',
                                                    }}
                                                >开启</button>
                                                <button
                                                    onClick={() => setState(false)}
                                                    style={{
                                                        padding: '2px 10px', borderRadius: 4, border: 'none',
                                                        cursor: 'pointer', fontSize: 10, fontWeight: 700,
                                                        background: state === false ? 'var(--red)' : 'rgba(239,68,68,0.12)',
                                                        color: state === false ? '#fff' : 'rgba(239,68,68,0.5)',
                                                        transition: 'all .1s',
                                                    }}
                                                >关闭</button>
                                            </div>
                                        </div>
                                    );
                                })}
                            </div>
                        </div>
                    )}

                    {/* 广播 */}
                    {selectedServer && (
                        <div className="card">
                            <div className="card-header" style={{ padding: '10px 14px' }}>
                                <div className="card-title" style={{ fontSize: 13 }}>📢 游戏广播</div>
                            </div>
                            <div className="card-body" style={{ padding: '10px 14px', display: 'flex', flexDirection: 'column', gap: 8 }}>
                                <input
                                    type="text"
                                    className="rcon-input"
                                    placeholder="输入广播内容..."
                                    value={broadcastMsg}
                                    onChange={e => setBroadcastMsg(e.target.value)}
                                    onKeyDown={e => e.key === 'Enter' && sendBroadcast()}
                                    style={{ fontSize: 12, padding: '8px 10px' }}
                                />
                                <button
                                    className="rcon-btn"
                                    onClick={sendBroadcast}
                                    disabled={!broadcastMsg}
                                    style={{ fontSize: 12, padding: '8px 14px', opacity: broadcastMsg ? 1 : 0.4 }}
                                >发送广播</button>
                            </div>
                        </div>
                    )}
                </div>

                {/* ═══ 右侧面板 ═══ */}
                <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                    <div className="card">
                        {/* 右侧主标签 */}
                        <div style={{ display: 'flex', borderBottom: '1px solid var(--border)' }}>
                            {([
                                { k: 'control' as const, label: '👥 玩家管理', hint: serverState?.players?.length || 0 },
                                { k: 'chat' as const, label: '💬 实时聊天', hint: chatMsgs.length },
                                { k: 'logs' as const, label: '📋 系统日志', hint: logs.length },
                            ]).map(t => (
                                <button
                                    key={t.k}
                                    onClick={() => setRightTab(t.k)}
                                    style={{
                                        flex: 1, padding: '12px', fontSize: 12, border: 'none', background: 'transparent',
                                        cursor: 'pointer', fontWeight: rightTab === t.k ? 600 : 400,
                                        color: rightTab === t.k ? 'var(--text)' : 'var(--text3)',
                                        borderBottom: rightTab === t.k ? '2px solid var(--text)' : '2px solid transparent',
                                        transition: 'all .12s', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 6,
                                    }}
                                >
                                    {t.label}
                                    {t.hint > 0 && (
                                        <span style={{ fontSize: 10, background: rightTab === t.k ? 'var(--text)' : 'var(--bg3)', color: rightTab === t.k ? 'var(--bg)' : 'var(--text3)', padding: '1px 6px', borderRadius: 10, fontWeight: 600 }}>
                                            {t.hint}
                                        </span>
                                    )}
                                </button>
                            ))}
                        </div>

                        {/* ═══ 玩家管理 ═══ */}
                        {rightTab === 'control' && selectedServer && (
                            <div>
                                {/* 子标签：玩家 / 封禁 / 警告 */}
                                <div style={{ display: 'flex', borderBottom: '1px solid var(--border)' }}>
                                    {[
                                        { k: 'players' as const, label: '玩家列表' },
                                        { k: 'bans' as const, label: '封禁记录' },
                                        { k: 'warns' as const, label: '警告记录' },
                                    ].map(t => (
                                        <button
                                            key={t.k}
                                            onClick={() => setActiveTab(t.k)}
                                            style={{
                                                padding: '8px 16px', fontSize: 11, border: 'none', background: 'transparent',
                                                cursor: 'pointer', fontWeight: activeTab === t.k ? 600 : 400,
                                                color: activeTab === t.k ? 'var(--text)' : 'var(--text3)',
                                                borderBottom: activeTab === t.k ? '2px solid var(--text)' : '2px solid transparent',
                                                transition: 'all .12s',
                                            }}
                                        >{t.label}</button>
                                    ))}
                                    <div style={{ flex: 1 }} />
                                    {serverStateLoading && <span style={{ fontSize: 10, color: 'var(--text3)', alignSelf: 'center', paddingRight: 12 }}>刷新中...</span>}
                                </div>

                                {/* 玩家列表 */}
                                {activeTab === 'players' && (serverState ? (
                                    <div>
                                        {/* 阵营标签 */}
                                        <div style={{ display: 'flex', background: 'var(--bg3)' }}>
                                            {(() => {
                                                const teams = serverState.teams?.length > 0
                                                    ? serverState.teams
                                                    : [{ team_id: 1, faction: '队伍 1' }, { team_id: 2, faction: '队伍 2' }];
                                                return [
                                                    ...teams,
                                                    { team_id: 0, faction: '未部署' },
                                                ].map((t: any) => {
                                                    const cnt = (serverState.players || []).filter((p: any) => p.team_id === t.team_id).length;
                                                    if (t.team_id === 0 && cnt === 0) return null;
                                                    const active = selectedTeam === t.team_id;
                                                    return (
                                                        <button
                                                            key={t.team_id}
                                                            onClick={() => setSelectedTeam(t.team_id)}
                                                            style={{
                                                                flex: 1, padding: '10px 12px', cursor: 'pointer', textAlign: 'center', fontSize: 12,
                                                                border: 'none', background: active ? 'var(--bg2)' : 'transparent',
                                                                color: active ? 'var(--text)' : 'var(--text3)',
                                                                borderBottom: active ? '2px solid var(--text)' : '2px solid transparent',
                                                                transition: 'all .12s', fontWeight: active ? 600 : 400,
                                                            }}
                                                        >
                                                            <span style={{ fontSize: 16, marginRight: 4 }}>{factionFlag(t.faction)}</span>
                                                            {t.faction} <span style={{ opacity: 0.5 }}>({cnt})</span>
                                                        </button>
                                                    );
                                                });
                                            })()}
                                        </div>

                                        {/* 小队列表 */}
                                        <div style={{ maxHeight: 420, overflowY: 'auto' }}>
                                            {[selectedTeam].map(teamId => {
                                                const tp = (serverState.players || []).filter((p: any) => p.team_id === teamId);
                                                const ts = (serverState.squads || []).filter((s: any) => s.team_id === teamId);
                                                const sp = (sid: string | null) => tp.filter((p: any) => p.squad_id === sid);
                                                const us = tp.filter((p: any) => !p.squad_id);
                                                const orphanSquadIds = new Set<string>();
                                                tp.forEach((p: any) => {
                                                    if (p.squad_id && !ts.some((s: any) => s.squad_id === p.squad_id))
                                                        orphanSquadIds.add(p.squad_id);
                                                });

                                                if (tp.length === 0) return <div key={teamId} style={{ padding: 40, textAlign: 'center', color: 'var(--text3)' }}>无玩家</div>;

                                                return (
                                                    <div key={teamId}>
                                                        {ts.map((sq: any) => {
                                                            const members = sp(sq.squad_id);
                                                            return (
                                                                <SquadBlock
                                                                    key={sq.name}
                                                                    squad={sq}
                                                                    members={members}
                                                                    onAction={execPlayerAction}
                                                                    onDisband={() => { if (confirm(`解散 ${sq.name}?`)) execDisbandSquad(teamId, sq.squad_id); }}
                                                                    adminSteamIds={serverState.admin_steam_ids}
                                                                />
                                                            );
                                                        })}
                                                        {Array.from(orphanSquadIds).map(sid => (
                                                            <SquadBlock
                                                                key={`orphan-${sid}`}
                                                                squad={{ name: `小队 ${sid}`, creator: '—', squad_id: sid }}
                                                                members={sp(sid)}
                                                                onAction={execPlayerAction}
                                                                onDisband={() => { if (confirm(`解散 小队 ${sid}?`)) execDisbandSquad(teamId, sid); }}
                                                                adminSteamIds={serverState.admin_steam_ids}
                                                            />
                                                        ))}
                                                        {us.length > 0 && (
                                                            <SquadBlock
                                                                squad={{ name: '未入队', creator: '', squad_id: null }}
                                                                members={us}
                                                                onAction={execPlayerAction}
                                                                onDisband={null}
                                                                adminSteamIds={serverState.admin_steam_ids}
                                                                collapsed={false}
                                                            />
                                                        )}
                                                    </div>
                                                );
                                            })}
                                        </div>
                                    </div>
                                ) : (
                                    <div style={{ padding: 40, textAlign: 'center', color: 'var(--text3)' }}>
                                        {serverStateLoading ? '加载中...' : '点击右上角 🔄 刷新获取玩家列表'}
                                    </div>
                                ))}

                                {/* 封禁列表 */}
                                {activeTab === 'bans' && (
                                    bans.length > 0 ? (
                                        <table style={{ fontSize: 12 }}>
                                            <thead><tr>
                                                <th style={{ padding: '10px 14px' }}>玩家</th>
                                                <th style={{ padding: '10px 14px' }}>时长</th>
                                                <th style={{ padding: '10px 14px' }}>原因</th>
                                                <th style={{ padding: '10px 14px' }}>管理员</th>
                                            </tr></thead>
                                            <tbody>{bans.map((b, i) => (
                                                <tr key={i}>
                                                    <td style={{ padding: '8px 14px', fontWeight: 500 }}>{b.player_name}</td>
                                                    <td style={{ padding: '8px 14px' }}><span className="badge red">{b.duration}</span></td>
                                                    <td style={{ padding: '8px 14px', color: 'var(--text2)' }}>{b.reason}</td>
                                                    <td style={{ padding: '8px 14px', color: 'var(--text3)' }}>{b.admin}</td>
                                                </tr>
                                            ))}</tbody>
                                        </table>
                                    ) : <div style={{ padding: 30, textAlign: 'center', color: 'var(--text3)' }}>✅ 无封禁记录</div>
                                )}

                                {/* 警告列表 */}
                                {activeTab === 'warns' && (
                                    warns.length > 0 ? (
                                        <table style={{ fontSize: 12 }}>
                                            <thead><tr>
                                                <th style={{ padding: '10px 14px' }}>玩家</th>
                                                <th style={{ padding: '10px 14px' }}>原因</th>
                                                <th style={{ padding: '10px 14px' }}>管理员</th>
                                            </tr></thead>
                                            <tbody>{warns.map((w, i) => (
                                                <tr key={i}>
                                                    <td style={{ padding: '8px 14px', fontWeight: 500 }}>{w.player_name}</td>
                                                    <td style={{ padding: '8px 14px', color: 'var(--text2)' }}>{w.reason}</td>
                                                    <td style={{ padding: '8px 14px', color: 'var(--text3)' }}>{w.admin}</td>
                                                </tr>
                                            ))}</tbody>
                                        </table>
                                    ) : <div style={{ padding: 30, textAlign: 'center', color: 'var(--text3)' }}>✅ 无警告记录</div>
                                )}
                            </div>
                        )}

                        {/* ═══ 实时聊天 ═══ */}
                        {rightTab === 'chat' && (
                            <div style={{ height: 500, overflowY: 'auto', padding: '10px 0', display: 'flex', flexDirection: 'column' }}>
                                {chatMsgs.length === 0 && <div style={{ color: 'var(--text3)', textAlign: 'center', padding: 60 }}>等待聊天消息...</div>}
                                {chatMsgs.map((c, i) => (
                                    <div key={i} style={{
                                        padding: '5px 14px', display: 'flex', gap: 8, alignItems: 'baseline',
                                        borderBottom: '1px solid var(--border)',
                                        transition: 'background .1s',
                                    }}>
                                        <span style={{ color: 'var(--text3)', fontSize: 10, fontFamily: 'monospace', flexShrink: 0, width: 72, textAlign: 'right' }}>
                                            {c.time.toLocaleTimeString()}
                                        </span>
                                        <span style={{
                                            fontSize: 9, fontWeight: 700, padding: '1px 5px', borderRadius: 3,
                                            background: (CHANNEL_COLORS[c.channel] || '#a78bfa') + '22',
                                            color: CHANNEL_COLORS[c.channel] || '#a78bfa',
                                            flexShrink: 0, textTransform: 'uppercase',
                                        }}>{c.channel}</span>
                                        <span style={{ fontWeight: 600, fontSize: 12, flexShrink: 0, color: 'var(--text)' }}>{c.player}</span>
                                        <span style={{ color: 'var(--text2)', fontSize: 12, wordBreak: 'break-word' }}>{c.message}</span>
                                    </div>
                                ))}
                                <div ref={chatEndRef} />
                            </div>
                        )}

                        {/* ═══ 系统日志 ═══ */}
                        {rightTab === 'logs' && (
                            <div style={{ height: 500, overflowY: 'auto', padding: '10px 0', fontFamily: "'JetBrains Mono', 'Fira Code', monospace", fontSize: 11 }}>
                                {logs.length === 0 && <div style={{ color: 'var(--text3)', textAlign: 'center', padding: 60 }}>等待日志...</div>}
                                {logs.map((entry, i) => (
                                    <div key={i} style={{
                                        padding: '3px 14px', display: 'flex', gap: 8, alignItems: 'baseline',
                                        borderBottom: '1px solid var(--border)',
                                        lineHeight: 1.7,
                                    }}>
                                        <span style={{ color: 'var(--text3)', flexShrink: 0, fontSize: 10 }}>
                                            [{new Date(entry.logged_at).toLocaleTimeString()}]
                                        </span>
                                        <span style={{
                                            color: LOG_LEVEL_COLORS[entry.log_level] || 'var(--text3)',
                                            flexShrink: 0, fontSize: 10, fontWeight: 600,
                                        }}>
                                            {entry.log_level}
                                        </span>
                                        <span style={{ color: 'var(--text2)', flexShrink: 0, fontSize: 10, opacity: 0.7 }}>
                                            [{entry.category || 'General'}]
                                        </span>
                                        <span style={{ color: 'var(--text2)', wordBreak: 'break-all' }}>{entry.message}</span>
                                    </div>
                                ))}
                                <div ref={logsEndRef} style={{ height: 1 }} />
                            </div>
                        )}
                    </div>
                </div>
            </div>

            {/* ═══ 删除确认 Modal ═══ */}
            {deleteTarget && (
                <Modal onClose={() => setDeleteTarget(null)}>
                    <h3 style={{ color: 'var(--red)', marginBottom: 12, fontSize: 16 }}>⚠️ 删除服务器</h3>
                    <p style={{ color: 'var(--text2)', fontSize: 13, marginBottom: 16, lineHeight: 1.6 }}>
                        确定删除 <strong style={{ color: 'var(--text)' }}>{deleteTarget.name}</strong>？此操作不可撤销。
                    </p>
                    {deleteError && <div style={{ color: 'var(--red)', marginBottom: 12, fontSize: 12 }}>{deleteError}</div>}
                    <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
                        <button onClick={() => setDeleteTarget(null)} disabled={deleting} style={{ padding: '8px 18px', background: 'var(--bg3)', border: '1px solid var(--border)', borderRadius: 6, color: 'var(--text2)', cursor: 'pointer', fontSize: 12 }}>
                            取消
                        </button>
                        <button onClick={handleConfirmDelete} disabled={deleting} style={{ padding: '8px 18px', background: 'var(--red)', border: 'none', borderRadius: 6, color: '#fff', cursor: 'pointer', fontSize: 12, fontWeight: 600, opacity: deleting ? 0.5 : 1 }}>
                            {deleting ? '删除中...' : '确认删除'}
                        </button>
                    </div>
                </Modal>
            )}

            {/* ═══ 添加服务器 Modal ═══ */}
            {showAddModal && (
                <Modal onClose={() => setShowAddModal(false)}>
                    <h3 style={{ marginBottom: 18, fontSize: 16 }}>🖥️ 添加游戏服务器</h3>
                    {newToken ? (
                        <div>
                            <div className="terminal" style={{ padding: 16, marginBottom: 14, wordBreak: 'break-all', fontSize: 13, borderColor: 'rgba(34,197,94,0.3)' }}>
                                <div style={{ color: '#22c55e', fontWeight: 600, marginBottom: 8 }}>✅ 服务器已添加</div>
                                <div style={{ color: 'var(--text3)', fontSize: 11, marginBottom: 4 }}>Agent Token（请在游戏服务器 .env 中配置）：</div>
                                <code style={{ color: '#22c55e', fontSize: 12 }}>{newToken}</code>
                            </div>
                            <button className="rcon-btn" onClick={() => setShowAddModal(false)}>关闭</button>
                        </div>
                    ) : (
                        <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
                            <input className="rcon-input" placeholder="服务器名称" value={form.name} onChange={e => setForm({ ...form, name: e.target.value })} />
                            <input className="rcon-input" placeholder="服务器 IP" value={form.ip} onChange={e => setForm({ ...form, ip: e.target.value })} />
                            <div style={{ display: 'flex', gap: 10 }}>
                                <input className="rcon-input" type="number" placeholder="RCON 端口" value={form.rcon_port || ''} onChange={e => setForm({ ...form, rcon_port: parseInt(e.target.value) || 0 })} style={{ flex: 1 }} />
                                <input className="rcon-input" type="password" placeholder="RCON 密码" value={form.rcon_password} onChange={e => setForm({ ...form, rcon_password: e.target.value })} style={{ flex: 2 }} />
                            </div>
                            {error && <div style={{ color: 'var(--red)', fontSize: 12 }}>{error}</div>}
                            <div style={{ display: 'flex', gap: 8, marginTop: 4 }}>
                                <button className="rcon-btn" onClick={handleAddServer} disabled={submitting}>{submitting ? '验证中...' : '验证并添加'}</button>
                                <button onClick={() => setShowAddModal(false)} style={{ padding: '10px 16px', background: 'var(--bg3)', border: '1px solid var(--border)', borderRadius: 6, color: 'var(--text2)', cursor: 'pointer', fontSize: 13 }}>取消</button>
                            </div>
                        </div>
                    )}
                </Modal>
            )}
        </div>
    );
}

/* ═══ 子组件 ═══ */

function InfoRow({ label, value }: { label: string; value: string }) {
    return (
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', borderBottom: '1px solid var(--border)', paddingBottom: 6 }}>
            <span style={{ color: 'var(--text3)', fontSize: 11 }}>{label}</span>
            <span style={{ fontWeight: 600, fontSize: 12 }}>{value}</span>
        </div>
    );
}

function SquadBlock({ squad, members, onAction, onDisband, adminSteamIds, collapsed: forceCollapsed }: {
    squad: any; members: any[]; onAction: (name: string, action: string, msg?: string) => void;
    onDisband: (() => void) | null; adminSteamIds?: string[]; collapsed?: boolean;
}) {
    const [collapsed, setCollapsed] = useState(forceCollapsed ?? (members.length > 8));
    const leader = members.find((m: any) => m.is_leader);

    return (
        <div style={{ borderBottom: '1px solid var(--border)' }}>
            <div
                onClick={() => setCollapsed(!collapsed)}
                style={{
                    display: 'flex', justifyContent: 'space-between', alignItems: 'center',
                    padding: '8px 14px', background: 'var(--bg3)', cursor: 'pointer',
                    userSelect: 'none', transition: 'background .1s',
                    fontSize: 12,
                }}
                onMouseEnter={e => { e.currentTarget.style.background = 'var(--bg4)'; }}
                onMouseLeave={e => { e.currentTarget.style.background = 'var(--bg3)'; }}
            >
                <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                    <span style={{ fontSize: 10, color: 'var(--text3)', transition: 'transform .15s', transform: collapsed ? 'rotate(-90deg)' : 'rotate(0)' }}>▼</span>
                    <strong>{squad.name}</strong>
                    {leader && <span style={{ fontSize: 10, color: '#f59e0b' }}>👑 {leader.name}</span>}
                </div>
                <div style={{ display: 'flex', gap: 10, alignItems: 'center' }}>
                    <span style={{ fontSize: 10, color: 'var(--text3)' }}>{squad.creator || ''}</span>
                    <span style={{ fontSize: 10, color: 'var(--text3)', background: 'var(--bg2)', padding: '1px 7px', borderRadius: 10 }}>{members.length}</span>
                    {onDisband && (
                        <span
                            onClick={e => { e.stopPropagation(); onDisband(); }}
                            style={{ fontSize: 10, cursor: 'pointer', color: 'var(--red)', padding: '2px 6px', borderRadius: 4, background: 'rgba(239,68,68,0.08)' }}
                            title="解散小队"
                        >解散</span>
                    )}
                </div>
            </div>
            {!collapsed && members.length > 0 && (
                <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 11 }}>
                    <thead>
                        <tr style={{ background: 'var(--bg2)' }}>
                            <th style={{ padding: '5px 14px', color: 'var(--text3)', fontWeight: 500, textAlign: 'left', fontSize: 10 }}>玩家</th>
                            <th style={{ padding: '5px 6px', color: 'var(--text3)', fontWeight: 500, textAlign: 'left', fontSize: 10 }}>职业</th>
                            <th style={{ padding: '5px 14px', color: 'var(--text3)', fontWeight: 500, textAlign: 'right', fontSize: 10 }}>操作</th>
                        </tr>
                    </thead>
                    <tbody>
                        {members.map((p: any) => (
                            <tr key={p.name + (p.steam_id || '')} style={{ borderBottom: '1px solid var(--border)', transition: 'background .1s' }}
                                onMouseEnter={e => { e.currentTarget.style.background = 'var(--bg3)'; }}
                                onMouseLeave={e => { e.currentTarget.style.background = 'transparent'; }}
                            >
                                <td style={{ padding: '5px 14px' }}>
                                    <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
                                        <span style={{ fontWeight: 600, fontSize: 12 }}>{p.name}</span>
                                        {(p.is_admin || (adminSteamIds && p.steam_id && adminSteamIds.includes(p.steam_id))) && <span style={{ color: '#f59e0b', fontSize: 9, background: 'rgba(245,158,11,0.15)', padding: '1px 5px', borderRadius: 3, fontWeight: 700, letterSpacing: '0.02em' }}>OP</span>}
                                        {p.is_leader && <span style={{ color: '#f59e0b', fontSize: 9 }}>👑</span>}
                                    </div>
                                </td>
                                <td style={{ padding: '5px 6px', color: 'var(--text2)', fontSize: 10 }}>{p.role}</td>
                                <td style={{ padding: '5px 14px', textAlign: 'right' }}>
                                    <div style={{ display: 'flex', gap: 3, justifyContent: 'flex-end' }}>
                                        <ActionBtn color="var(--text2)" bg="var(--bg4)" onClick={() => onAction(p.name, 'warn')}>警告</ActionBtn>
                                        <ActionBtn color="var(--red)" bg="rgba(239,68,68,0.08)" onClick={() => { if (confirm(`踢出 ${p.name}?`)) onAction(p.name, 'kick', '管理员操作'); }}>踢出</ActionBtn>
                                        <ActionBtn color="var(--red)" bg="rgba(239,68,68,0.12)" onClick={() => { if (confirm(`封禁 ${p.name}?`)) onAction(p.name, 'ban', '管理员操作'); }}>封禁</ActionBtn>
                                        <ActionBtn color="var(--blue)" bg="rgba(59,130,246,0.08)" onClick={() => { if (confirm(`强制 ${p.name} 跳边?`)) onAction(p.name, 'team_change'); }}>跳边</ActionBtn>
                                    </div>
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            )}
        </div>
    );
}

function ActionBtn({ children, onClick, color, bg }: { children: string; onClick: () => void; color: string; bg: string }) {
    const [hover, setHover] = useState(false);
    return (
        <span
            onClick={onClick}
            onMouseEnter={() => setHover(true)}
            onMouseLeave={() => setHover(false)}
            style={{
                cursor: 'pointer', fontSize: 9, fontWeight: 600,
                padding: '3px 7px', borderRadius: 4,
                background: hover ? color : bg,
                color: hover ? '#fff' : color,
                transition: 'all .12s', whiteSpace: 'nowrap',
                border: `1px solid ${hover ? color : 'transparent'}`,
            }}
        >{children}</span>
    );
}

function Modal({ children, onClose }: { children: React.ReactNode; onClose: () => void }) {
    return (
        <div
            style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.7)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 1000, backdropFilter: 'blur(4px)' }}
            onClick={e => { if (e.target === e.currentTarget) onClose(); }}
        >
            <div style={{ background: 'var(--bg2)', border: '1px solid var(--border)', borderRadius: 12, padding: 24, width: 460, maxWidth: '90vw', boxShadow: '0 20px 60px rgba(0,0,0,0.5)' }} onClick={e => e.stopPropagation()}>
                {children}
            </div>
        </div>
    );
}
