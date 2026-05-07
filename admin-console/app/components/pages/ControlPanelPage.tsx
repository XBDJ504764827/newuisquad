'use client';

import { useState, useEffect, useRef, useCallback } from 'react';
import { useServers } from '../../lib/useServers';
import { api } from '../../lib/api';
import { ServerInfoBar, factionFlag } from './ControlPanel/ServerInfoBar';
import type { ServerInfo as SvrInfo, ServerState, ServerInfoDisplay, PlayerState } from '../../types';
import { LeftSidePanel } from './ControlPanel/LeftSidePanel';
import { SquadBlock } from './ControlPanel/SquadBlock';
import { Modal } from './ControlPanel/Modal';

interface LogEntry { log_level: string; category: string | null; message: string; raw_line: string | null; logged_at: string; }
interface ChatMsg { time: Date; player: string; message: string; channel: string; }
interface BanEntry { player_name: string; steam_id: string; duration: string; reason: string; admin: string; }
interface WarnEntry { player_name: string; steam_id: string; reason: string; admin: string; }
interface ServerInfoLocal { server_name: string; player_count: number; max_players: number; map_name: string; game_mode: string; next_map: string; next_layer: string; }

export default function ControlPanelPage() {
    const { servers } = useServers();
    const [selectedServer, setSelectedServer] = useState<SvrInfo | null>(null);
    const [showAddModal, setShowAddModal] = useState(false);
    const [logs, setLogs] = useState<LogEntry[]>([]);
    const [chatMsgs, setChatMsgs] = useState<ChatMsg[]>([]);
    const [notifications, setNotifications] = useState<Array<{ id: number; text: string; type: string }>>([]);
    const [rconCommand, setRconCommand] = useState('');
    const [rconResult, setRconResult] = useState('');
    const [loading, setLoading] = useState(true);
    const wsRef = useRef<WebSocket | null>(null);
    const reconnectTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
    const reconnectAttemptRef = useRef(0);
    const logsEndRef = useRef<HTMLDivElement>(null);
    const chatEndRef = useRef<HTMLDivElement>(null);
    const notifId = useRef(0);
    const prevMapRef = useRef<string>('');

    const [form, setForm] = useState({ name: '', ip: '', rcon_port: 28016, rcon_password: '', admin_user: 'Admin' });
    const [submitting, setSubmitting] = useState(false);
    const [newToken, setNewToken] = useState('');
    const [error, setError] = useState('');
    const [deleting, setDeleting] = useState(false);
    const [deleteTarget, setDeleteTarget] = useState<any>(null);
    const [deleteError, setDeleteError] = useState('');
    const [serverState, setServerState] = useState<ServerState | null>(null);
    const [serverStateLoading, setServerStateLoading] = useState(false);
    const [actionMsg, setActionMsg] = useState('');
    // 暖服作弊开关状态: null=未知, true=开启, false=关闭
    const [warmupToggles, setWarmupToggles] = useState<Record<string, boolean | null>>(() => {
        try { const v = localStorage.getItem('warmupToggles'); if (v) return JSON.parse(v); } catch {}
        return {};
    });
    const [activeTab, setActiveTab] = useState<'players' | 'bans' | 'warns'>('players');
    const [autoRefresh, setAutoRefresh] = useState(true);
    const [bans, setBans] = useState<BanEntry[]>([]);
    const [warns, setWarns] = useState<WarnEntry[]>([]);
    const [serverInfo, setServerInfo] = useState<ServerInfoLocal | null>(null);
    const [broadcastMsg, setBroadcastMsg] = useState('');
    const [rightTab, setRightTab] = useState<'control' | 'chat' | 'logs'>('control');
    const [banTarget, setBanTarget] = useState<any>(null);
    const [banDuration, setBanDuration] = useState(60);
    const [banReason, setBanReason] = useState('');
    const [slomoValue, setSlomoValue] = useState(1);

    useEffect(() => {
        if (servers.length > 0 && !selectedServer) setSelectedServer(servers[0]);
        setLoading(false);
    }, [servers, selectedServer]);

    useEffect(() => {
        if (!selectedServer) return;

        // 清除之前的重连定时器
        if (reconnectTimerRef.current) {
            clearTimeout(reconnectTimerRef.current);
            reconnectTimerRef.current = null;
        }
        // 关闭旧连接
        if (wsRef.current) {
            wsRef.current.onclose = null;
            wsRef.current.onerror = null;
            wsRef.current.close();
        }

        let cancelled = false;
        reconnectAttemptRef.current = 0;

        function connect() {
            if (cancelled) return;
            const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
            const token = localStorage.getItem('token') || '';
            const srv = selectedServer!;
            const ws = new WebSocket(`${proto}//${window.location.host}/api/v1/servers/${srv.id}/logs/stream?token=${encodeURIComponent(token)}`);

            ws.onopen = () => {
                reconnectAttemptRef.current = 0;
            };

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

            ws.onclose = () => {
                if (cancelled) return;
                const attempt = reconnectAttemptRef.current;
                const delay = Math.min(1000 * Math.pow(2, attempt), 30000);
                reconnectAttemptRef.current = attempt + 1;
                reconnectTimerRef.current = setTimeout(connect, delay);
            };

            ws.onerror = () => {
                ws.close();
            };

            wsRef.current = ws;
        }

        connect();
        setLogs([]);
        setChatMsgs([]);

        return () => {
            cancelled = true;
            if (reconnectTimerRef.current) {
                clearTimeout(reconnectTimerRef.current);
                reconnectTimerRef.current = null;
            }
            if (wsRef.current) {
                wsRef.current.onclose = null;
                wsRef.current.onerror = null;
                wsRef.current.close();
            }
        };
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

    // 换图时重置暖服开关（服务器端作弊已自动重置）
    useEffect(() => {
        const map = serverState?.map_name;
        if (map && map !== 'Unknown' && prevMapRef.current && prevMapRef.current !== map) {
            setWarmupToggles({});
            try { localStorage.removeItem('warmupToggles'); } catch {}
        }
        if (map) prevMapRef.current = map;
    }, [serverState?.map_name]);

    useEffect(() => {
        if (!autoRefresh || !selectedServer) return;
        let timer: ReturnType<typeof setInterval> | null = null;

        function startPolling() {
            if (timer) return;
            timer = setInterval(() => { fetchServerState(); fetchBansWarns(); }, 3000);
        }

        function stopPolling() {
            if (timer) { clearInterval(timer); timer = null; }
        }

        function onVisibilityChange() {
            if (document.hidden) {
                stopPolling();
            } else {
                fetchServerState(); fetchBansWarns(); // 恢复时立即刷新
                startPolling();
            }
        }

        document.addEventListener('visibilitychange', onVisibilityChange);
        startPolling();

        return () => {
            stopPolling();
            document.removeEventListener('visibilitychange', onVisibilityChange);
        };
    }, [autoRefresh, selectedServer, fetchServerState, fetchBansWarns]);

    const sendRcon = useCallback(async (cmd?: string) => {
        const command = cmd || rconCommand;
        if (!selectedServer || !command) return;
        const res = await api(`/servers/${selectedServer.id}/rcon`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ command, admin_user: 'Admin' }) });
        const data = await res.json();
        setRconResult(data.response || data.error || 'OK');
        if (!cmd) setRconCommand('');
    }, [selectedServer, rconCommand]);

    const execPlayerAction = useCallback(async (playerName: string, action: string, msg?: string, playerId?: number, duration?: number) => {
        if (!selectedServer) return;
        try {
            const body: any = { player_name: playerName, action, message: msg || '', admin_user: 'Admin' };
            if (playerId !== undefined) body.player_id = playerId;
            if (duration !== undefined) body.duration = duration;
            const res = await api(`/servers/${selectedServer.id}/player-action`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
            const data = await res.json();
            setActionMsg(data.error ? `失败: ${data.error}` : `${action} ${playerName} 成功`);
            setTimeout(() => setActionMsg(''), 3000);
            fetchServerState();
        } catch { setActionMsg('请求失败'); }
    }, [selectedServer, fetchServerState]);

    const handleBan = useCallback(async () => {
        if (!banTarget) return;
        await execPlayerAction(banTarget.name, 'ban', banReason || '管理员封禁', banTarget.player_id, banDuration);
        setBanTarget(null);
        setBanReason('');
    }, [banTarget, banReason, banDuration, execPlayerAction]);

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

    if (loading) return <div className="page-view"><div className="card"><div className="card-body" style={{ textAlign: 'center', color: 'var(--text3)', padding: 48 }}>加载中...</div></div></div>;
    if (servers.length === 0 && !showAddModal) return <div className="page-view"><div className="card"><div className="empty-state"><h3 style={{ fontSize: 18, marginBottom: 8 }}>暂无服务器</h3><p style={{ color: 'var(--text3)', marginBottom: 20 }}>添加游戏服务器以开始管理</p><button className="rcon-btn" style={{ width: 'auto', paddingLeft: 24, paddingRight: 24 }} onClick={() => { setShowAddModal(true); setNewToken(''); setError(''); }}>+ 添加服务器</button></div></div></div>;

    return (
        <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
            {/* ═══ 顶栏：服务器选择 + 信息条 ═══ */}
            <ServerInfoBar
                servers={servers}
                selectedServer={selectedServer}
                autoRefresh={autoRefresh}
                serverInfo={serverInfo}
                serverState={serverState}
                onSelectServer={setSelectedServer}
                onAddServer={() => { setShowAddModal(true); setNewToken(''); setError(''); }}
                onToggleAutoRefresh={() => setAutoRefresh(!autoRefresh)}
                onManualRefresh={() => { fetchServerState(); fetchBansWarns(); }}
            />

            {/* ═══ 通知区域 ═══ */}
            {notifications.map(n => (
                <div key={n.id} className="badge" style={{
                    padding: '8px 14px', fontSize: 12, borderRadius: 6, animation: 'fadeIn .3s', fontWeight: 500,
                    background: n.type === 'join' ? 'rgba(34,197,94,0.1)' : 'rgba(239,68,68,0.1)',
                    color: n.type === 'join' ? '#22c55e' : 'var(--red)',
                    border: `1px solid ${n.type === 'join' ? 'rgba(34,197,94,0.2)' : 'rgba(239,68,68,0.2)'}`,
                }}>{n.type === 'join' ? '✅' : '👋'} {n.text}</div>
            ))}

            {actionMsg && (
                <div style={{ padding: '8px 14px', fontSize: 12, borderRadius: 6, background: actionMsg.includes('失败') ? 'rgba(239,68,68,0.08)' : 'rgba(34,197,94,0.08)', color: actionMsg.includes('失败') ? 'var(--red)' : '#22c55e', border: `1px solid ${actionMsg.includes('失败') ? 'rgba(239,68,68,0.15)' : 'rgba(34,197,94,0.15)'}`, fontWeight: 500 }}>
                    {actionMsg}
                </div>
            )}

            {/* ═══ 主内容区 ═══ */}
            <div style={{ display: 'grid', gridTemplateColumns: '300px 1fr', gap: 16, alignItems: 'start' }}>
                {/* ═══ 左侧面板 ═══ */}
                <LeftSidePanel
                    selectedServer={selectedServer}
                    rconCommand={rconCommand}
                    rconResult={rconResult}
                    broadcastMsg={broadcastMsg}
                    warmupToggles={warmupToggles}
                    slomoValue={slomoValue}
                    onRconCommandChange={setRconCommand}
                    onSendRcon={sendRcon}
                    onDeleteServer={handleDeleteClick}
                    onBroadcastMsgChange={setBroadcastMsg}
                    onSendBroadcast={sendBroadcast}
                    onSlomoChange={setSlomoValue}
                    onWarmupToggle={(key: string, v: boolean) => {
                        const item = [
                            { key: 'novehicleclaim', cmd: 'AdminDisableVehicleClaiming' },
                            { key: 'forcevehicle', cmd: 'AdminForceAllVehicleAvailability' },
                            { key: 'forcedeploy', cmd: 'AdminForceAllDeployableAvailability' },
                            { key: 'forcerole', cmd: 'AdminForceAllRoleAvailability' },
                            { key: 'noenemylimit', cmd: 'AdminDisableVehicleTeamRequirement' },
                            { key: 'nokitreq', cmd: 'AdminDisableVehicleKitRequirement' },
                            { key: 'norespawn', cmd: 'AdminNoRespawnTimer' },
                        ].find(it => it.key === key);
                        if (item) sendRcon(`${item.cmd} ${v ? 1 : 0}`);
                        const next = { ...warmupToggles, [key]: v };
                        setWarmupToggles(next);
                        try { localStorage.setItem('warmupToggles', JSON.stringify(next)); } catch {}
                    }}
                />

                {/* ═══ 右侧面板 ═══ */}
                <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                    <div className="card">
                        {/* 玩家管理 */}
                        <div style={{ padding: '6px 14px', borderBottom: '1px solid var(--border)', display: 'flex', alignItems: 'center', gap: 8, background: 'var(--bg3)' }}>
                            <span style={{ fontSize: 13, fontWeight: 600 }}>👥 玩家管理</span>
                            <span style={{ fontSize: 10, color: 'var(--text3)' }}>({serverState?.players?.length || 0}人在线)</span>
                            <div style={{ flex: 1 }} />
                            {serverStateLoading && <span style={{ fontSize: 10, color: 'var(--text3)' }}>刷新中...</span>}
                        </div>
                        {selectedServer && (
                            <div>
                                {/* 玩家列表 — 双方阵营左右排列 */}
                                {serverState ? (
                                    <div style={{ display: 'flex' }}>
                                        {(() => {
                                            const teams = serverState.teams?.length > 0
                                                ? serverState.teams
                                                : [{ team_id: 1, faction: '队伍 1' }, { team_id: 2, faction: '队伍 2' }];
                                            // 只显示 team 1 和 2 左右排列，未部署玩家在下方
                                            const mainTeams = teams.filter((t: any) => t.team_id === 1 || t.team_id === 2);
                                            const unassigned = (serverState.players || []).filter((p: any) => p.team_id === 0 || p.team_id === 0);
                                            return (
                                                <>
                                                    {mainTeams.map((team: any, idx: number) => {
                                                        const teamId = team.team_id;
                                                        const tp = (serverState.players || []).filter((p: any) => p.team_id === teamId);
                                                        const cnt = tp.length;
                                                        const ts = (serverState.squads || []).filter((s: any) => s.team_id === teamId);
                                                        const sp = (sid: string | null) => tp.filter((p: any) => p.squad_id === sid);
                                                        const us = tp.filter((p: any) => !p.squad_id);
                                                        const orphanSquadIds = new Set<string>();
                                                        tp.forEach((p: any) => {
                                                            if (p.squad_id && !ts.some((s: any) => s.squad_id === p.squad_id))
                                                                orphanSquadIds.add(p.squad_id);
                                                        });

                                                        return (
                                                            <div key={teamId} style={{
                                                                flex: 1, overflowY: 'auto',
                                                                borderLeft: idx === 1 ? '1px solid var(--border)' : 'none',
                                                            }}>
                                                                <div style={{
                                                                    padding: '8px 12px', background: 'var(--bg3)',
                                                                    display: 'flex', alignItems: 'center', gap: 6,
                                                                    borderBottom: '1px solid var(--border)',
                                                                    position: 'sticky', top: 0, zIndex: 1,
                                                                }}>
                                                                    <span style={{ fontSize: 14 }}>{factionFlag(team.faction)}</span>
                                                                    <span style={{ fontSize: 12, fontWeight: 600, color: 'var(--text)' }}>{team.faction}</span>
                                                                    <span style={{ fontSize: 10, color: 'var(--text3)' }}>({cnt}人)</span>
                                                                </div>
                                                                {tp.length === 0 ? (
                                                                    <div style={{ padding: 20, textAlign: 'center', color: 'var(--text3)', fontSize: 11 }}>暂无玩家</div>
                                                                ) : (
                                                                    <>
                                                                        {ts.map((sq: any) => {
                                                                            const members = sp(sq.squad_id);
                                                                            return (
                                                                                <SquadBlock
                                                                                    key={sq.name}
                                                                                    squad={sq}
                                                                                    members={members}
                                                                                    onAction={execPlayerAction}
                                                                                    onBan={setBanTarget}
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
                                                                                onBan={setBanTarget}
                                                                                onDisband={() => { if (confirm(`解散 小队 ${sid}?`)) execDisbandSquad(teamId, sid); }}
                                                                                adminSteamIds={serverState.admin_steam_ids}
                                                                            />
                                                                        ))}
                                                                        {us.length > 0 && (
                                                                            <SquadBlock
                                                                                squad={{ name: '未入队', creator: '', squad_id: null }}
                                                                                members={us}
                                                                                onAction={execPlayerAction}
                                                                                onBan={setBanTarget}
                                                                                onDisband={null}
                                                                                adminSteamIds={serverState.admin_steam_ids}
                                                                                collapsed={false}
                                                                            />
                                                                        )}
                                                                    </>
                                                                )}
                                                            </div>
                                                        );
                                                    })}
                                                </>
                                            );
                                        })()}
                                    </div>
                                ) : (
                                    <div style={{ padding: 40, textAlign: 'center', color: 'var(--text3)' }}>
                                        {serverStateLoading ? '加载中...' : '点击右上角 🔄 刷新获取玩家列表'}
                                    </div>
                                )}

                            </div>
                        )}
                    </div>
                </div>
            </div>

            {/* ═══ 封禁对话框 ═══ */}
            {banTarget && (
                <Modal onClose={() => { setBanTarget(null); setBanReason(''); }}>
                    <h3 style={{ marginBottom: 16, fontSize: 16 }}>⛔ 封禁玩家</h3>
                    <div style={{ marginBottom: 12 }}>
                        <span style={{ color: 'var(--text3)', fontSize: 12 }}>玩家：</span>
                        <strong style={{ fontSize: 14 }}>{banTarget.name}</strong>
                        <span style={{ color: 'var(--text3)', fontSize: 11, marginLeft: 8, fontFamily: 'monospace' }}>ID: {banTarget.player_id}</span>
                    </div>
                    <div style={{ marginBottom: 14 }}>
                        <span style={{ color: 'var(--text3)', fontSize: 12, display: 'block', marginBottom: 6 }}>封禁时长：</span>
                        <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap' }}>
                            {[
                                { label: '1小时', val: 60 },
                                { label: '6小时', val: 360 },
                                { label: '1天', val: 1440 },
                                { label: '3天', val: 4320 },
                                { label: '7天', val: 10080 },
                                { label: '永久', val: 0 },
                            ].map(opt => (
                                <button
                                    key={opt.val}
                                    onClick={() => setBanDuration(opt.val)}
                                    style={{
                                        padding: '6px 14px', fontSize: 12, borderRadius: 6, cursor: 'pointer', border: '1px solid',
                                        background: banDuration === opt.val ? 'var(--red)' : 'transparent',
                                        color: banDuration === opt.val ? '#fff' : 'var(--text2)',
                                        borderColor: banDuration === opt.val ? 'var(--red)' : 'var(--border)',
                                        fontWeight: banDuration === opt.val ? 600 : 400,
                                    }}
                                >{opt.label}</button>
                            ))}
                        </div>
                    </div>
                    <div style={{ marginBottom: 16 }}>
                        <span style={{ color: 'var(--text3)', fontSize: 12, display: 'block', marginBottom: 6 }}>封禁原因：</span>
                        <input
                            className="rcon-input"
                            placeholder="输入封禁原因（选填）"
                            value={banReason}
                            onChange={e => setBanReason(e.target.value)}
                            onKeyDown={e => { if (e.key === 'Enter') handleBan(); }}
                            autoFocus
                        />
                    </div>
                    <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
                        <button onClick={() => { setBanTarget(null); setBanReason(''); }} style={{ padding: '8px 18px', background: 'var(--bg3)', border: '1px solid var(--border)', borderRadius: 6, color: 'var(--text2)', cursor: 'pointer', fontSize: 12 }}>取消</button>
                        <button onClick={handleBan} style={{ padding: '8px 18px', background: 'var(--red)', border: 'none', borderRadius: 6, color: '#fff', cursor: 'pointer', fontSize: 12, fontWeight: 600 }}>确认封禁</button>
                    </div>
                </Modal>
            )}

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

