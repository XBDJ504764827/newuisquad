'use client';

import { useState, useEffect, useCallback, useRef } from 'react';
import { api } from '../../lib/api';

interface Server { id: number; server_id: string; name: string; ip: string; rcon_port: number; }
interface ChatMessage { id: number; player_name: string; steam_id: string; message: string; chat_type: string; logged_at: string; }
interface ConnectionEvent { id: number; player_name: string; steam_id: string; action: string; ip_address: string; logged_at: string; }
interface TeamkillEvent { id: number; attacker_name: string; victim_name: string; weapon: string; damage: number; logged_at: string; }
type TabKey = 'chat' | 'connections' | 'teamkills';

function formatTime(ts: string): string {
  const d = new Date(ts);
  const now = new Date();
  const diff = now.getTime() - d.getTime();
  if (diff < 60000) return '刚刚';
  if (diff < 3600000) return `${Math.floor(diff / 60000)} 分钟前`;
  if (d.toDateString() === now.toDateString()) return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  return d.toLocaleString([], { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
}

function chatTypeBadge(t: string): { bg: string; color: string; label: string } {
  switch (t?.toLowerCase()) {
    case 'chatall': return { bg: 'rgba(59,130,246,0.15)', color: '#3b82f6', label: '全部' };
    case 'chatteam': return { bg: 'rgba(34,197,94,0.15)', color: '#22c55e', label: '队伍' };
    case 'chatsquad': return { bg: 'rgba(234,179,8,0.15)', color: '#eab308', label: '小队' };
    case 'chatadmin': return { bg: 'rgba(239,68,68,0.15)', color: '#ef4444', label: '管理' };
    default: return { bg: 'rgba(156,163,175,0.15)', color: '#9ca3af', label: t || '未知' };
  }
}

export default function ServerFeedsPage() {
  const [servers, setServers] = useState<Server[]>([]);
  const [serverId, setServerId] = useState<number | null>(null);
  const [activeTab, setActiveTab] = useState<TabKey>('chat');
  const [chatMessages, setChatMessages] = useState<ChatMessage[]>([]);
  const [connections, setConnections] = useState<ConnectionEvent[]>([]);
  const [teamkills, setTeamkills] = useState<TeamkillEvent[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [wsStatus, setWsStatus] = useState<'connecting' | 'connected' | 'disconnected'>('disconnected');
  const wsRef = useRef<WebSocket | null>(null);
  const pollRef = useRef<ReturnType<typeof setInterval> | undefined>(undefined);
  const chatRef = useRef<HTMLDivElement>(null);
  const connRef = useRef<HTMLDivElement>(null);
  const tkRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    api('/servers').then(r => r.json()).then(d => {
      const list = d.data || [];
      setServers(list);
      if (list.length > 0) setServerId(list[0].id);
    }).catch(() => {});
  }, []);

  const loadHistorical = useCallback(async (sid: number) => {
    setLoading(true);
    try {
      const [chatRes, killRes, matchRes] = await Promise.all([
        api(`/servers/${sid}/chat-messages?limit=80`),
        api(`/servers/${sid}/kill-events?limit=40`),
        api(`/servers/${sid}/match-events?limit=30`),
      ]);
      const chatData = await chatRes.json();
      const killData = await killRes.json();
      const matchData = await matchRes.json();

      setChatMessages((chatData.data || []).map((c: any) => ({
        id: c.id, player_name: c.player_name || c.name || '', steam_id: c.steam_id || '',
        message: c.message || '', chat_type: c.chat_type || 'ChatAll', logged_at: c.logged_at || '',
      })));

      const connEvents = (matchData.data || []).filter((e: any) =>
        e.event_type === 'connect' || e.event_type === 'disconnect' || e.event_type === 'join' || e.event_type === 'leave'
      ).map((e: any) => ({
        id: e.id, player_name: e.player_name || e.name || '', steam_id: e.steam_id || '',
        action: e.event_type === 'connect' || e.event_type === 'join' ? 'connected' : 'disconnected',
        ip_address: e.ip || '', logged_at: e.logged_at || '',
      }));
      setConnections(connEvents);

      setTeamkills((killData.data || []).filter((e: any) => e.is_teamkill).map((e: any) => ({
        id: e.id, attacker_name: e.attacker_name || '', victim_name: e.victim_name || '',
        weapon: e.weapon || '', damage: e.damage || 0, logged_at: e.logged_at || '',
      })));
    } catch (e: any) {
      setError(e.message);
    }
    setLoading(false);
  }, []);

  useEffect(() => {
    if (serverId) loadHistorical(serverId);
  }, [serverId, loadHistorical]);

  // WebSocket for live feeds
  useEffect(() => {
    if (!serverId) return;
    const token = typeof window !== 'undefined' ? localStorage.getItem('token') : null;
    if (!token) return;
    const proto = window.location.protocol === 'https:' ? 'wss' : 'ws';
    const host = window.location.host;
    setWsStatus('connecting');
    const ws = new WebSocket(`${proto}://${host}/api/v1/servers/${serverId}/logs/stream?token=${token}`);
    wsRef.current = ws;

    ws.onopen = () => setWsStatus('connected');
    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        if (data.type === 'chat' || data.chat_type) {
          setChatMessages(prev => [...prev.slice(-199), {
            id: Date.now(), player_name: data.player_name || data.name || '',
            steam_id: data.steam_id || '', message: data.message || '',
            chat_type: data.chat_type || 'ChatAll', logged_at: new Date().toISOString(),
          }]);
        } else if (data.type === 'connect' || data.type === 'disconnect' || data.event_type === 'connect' || data.event_type === 'disconnect') {
          setConnections(prev => [...prev.slice(-99), {
            id: Date.now(), player_name: data.player_name || data.name || '',
            steam_id: data.steam_id || '', action: data.type === 'disconnect' || data.event_type === 'disconnect' ? 'disconnected' : 'connected',
            ip_address: data.ip || '', logged_at: new Date().toISOString(),
          }]);
        } else if (data.type === 'kill' && data.is_teamkill) {
          setTeamkills(prev => [...prev.slice(-99), {
            id: Date.now(), attacker_name: data.attacker_name || '',
            victim_name: data.victim_name || '', weapon: data.weapon || '',
            damage: data.damage || 0, logged_at: new Date().toISOString(),
          }]);
        }
      } catch {}
    };
    ws.onclose = () => setWsStatus('disconnected');
    ws.onerror = () => setWsStatus('disconnected');

    // Fallback polling
    pollRef.current = setInterval(async () => {
      if (ws.readyState === WebSocket.OPEN) return;
      try {
        const res = await api(`/servers/${serverId}/chat-messages?limit=10`);
        const d = await res.json();
        const latest = (d.data || []).slice(0, 5);
        if (latest.length > 0) {
          setChatMessages(prev => {
            const existing = new Set(prev.map(m => m.id));
            const news = latest.filter((m: any) => !existing.has(m.id)).map((m: any) => ({
              id: m.id, player_name: m.player_name || '', steam_id: m.steam_id || '',
              message: m.message || '', chat_type: m.chat_type || 'ChatAll', logged_at: m.logged_at || '',
            }));
            return news.length > 0 ? [...prev.slice(-195), ...news] : prev;
          });
        }
      } catch {}
    }, 10000);

    return () => {
      ws.close();
      wsRef.current = null;
      clearInterval(pollRef.current);
    };
  }, [serverId]);

  const containerRef = (tab: TabKey) => {
    if (tab === 'chat') return chatRef;
    if (tab === 'connections') return connRef;
    return tkRef;
  };

  const styles = {
    container: { padding: 20, display: 'flex', flexDirection: 'column' as const, height: 'calc(100vh - 100px)' },
    header: { display: 'flex', alignItems: 'center', gap: 12, marginBottom: 16, flexWrap: 'wrap' as const },
    select: { padding: '8px 12px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--bg2)', color: 'var(--text)', fontSize: 14, minWidth: 200 },
    btn: { padding: '6px 14px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--bg2)', color: 'var(--text)', cursor: 'pointer', fontSize: 12 },
    tabs: { display: 'flex', gap: 2, marginBottom: 0, borderBottom: '1px solid var(--border)' },
    tabBtn: (active: boolean) => ({ padding: '10px 20px', border: 'none', background: active ? 'var(--text)' : 'transparent', color: active ? 'var(--bg)' : 'var(--text2)', cursor: 'pointer', borderRadius: '8px 8px 0 0', fontSize: 13, fontWeight: active ? 600 : 400 }),
    feedList: { flex: 1, overflowY: 'auto' as const, padding: '12px 0' },
    feedItem: { padding: '10px 14px', borderRadius: 8, marginBottom: 8, background: 'var(--bg2)', border: '1px solid var(--border)' },
    badge: (bg: string, color: string) => ({ display: 'inline-block', padding: '2px 8px', borderRadius: 10, fontSize: 10, fontWeight: 600, background: bg, color, marginLeft: 8 }),
  };

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h2 style={{ margin: 0, fontSize: 20, fontWeight: 600 }}>实时信息</h2>
        <select value={serverId || ''} onChange={e => setServerId(Number(e.target.value))} style={styles.select}>
          {servers.map(s => <option key={s.id} value={s.id}>{s.name} ({s.ip}:{s.rcon_port})</option>)}
        </select>
        <span style={{ fontSize: 12, display: 'flex', alignItems: 'center', gap: 6 }}>
          <span style={{ width: 8, height: 8, borderRadius: 4, background: wsStatus === 'connected' ? '#10b981' : wsStatus === 'connecting' ? '#f59e0b' : '#ef4444' }} />
          {wsStatus === 'connected' ? '实时' : wsStatus === 'connecting' ? '连接中' : '轮询模式'}
        </span>
        <button onClick={() => serverId && loadHistorical(serverId)} style={{ ...styles.btn, background: 'var(--text)', color: 'var(--bg)' }}>刷新</button>
      </div>

      <div style={styles.tabs}>
        {(['chat', 'connections', 'teamkills'] as TabKey[]).map(t => (
          <button key={t} onClick={() => setActiveTab(t)} style={styles.tabBtn(activeTab === t)}>
            {t === 'chat' ? '聊天消息' : t === 'connections' ? '玩家连接' : '误伤队友'}
          </button>
        ))}
      </div>

      {error && <div style={{ padding: '8px 12px', background: 'rgba(239,68,68,0.1)', color: '#ef4444', borderRadius: 6, marginTop: 8, fontSize: 13 }}>{error}</div>}

      {/* Chat Feed */}
      {activeTab === 'chat' && (
        <div ref={chatRef} style={styles.feedList}>
          {loading && chatMessages.length === 0 && <div style={{ textAlign: 'center', padding: 30, color: 'var(--text3)' }}>加载中...</div>}
          {!loading && chatMessages.length === 0 && <div style={{ textAlign: 'center', padding: 30, color: 'var(--text3)' }}>暂无聊天消息</div>}
          {chatMessages.map(m => {
            const badge = chatTypeBadge(m.chat_type);
            return (
              <div key={m.id} style={styles.feedItem}>
                <div style={{ display: 'flex', alignItems: 'center', marginBottom: 4 }}>
                  <strong style={{ fontSize: 13 }}>{m.player_name}</strong>
                  <span style={styles.badge(badge.bg, badge.color)}>{badge.label}</span>
                  <span style={{ fontSize: 11, color: 'var(--text3)', marginLeft: 'auto' }}>{formatTime(m.logged_at)}</span>
                </div>
                <div style={{ fontSize: 13, color: 'var(--text)' }}>{m.message}</div>
              </div>
            );
          })}
        </div>
      )}

      {/* Connections Feed */}
      {activeTab === 'connections' && (
        <div ref={connRef} style={styles.feedList}>
          {loading && connections.length === 0 && <div style={{ textAlign: 'center', padding: 30, color: 'var(--text3)' }}>加载中...</div>}
          {!loading && connections.length === 0 && <div style={{ textAlign: 'center', padding: 30, color: 'var(--text3)' }}>暂无连接事件</div>}
          {connections.map(c => (
            <div key={c.id} style={{ ...styles.feedItem, borderLeft: c.action === 'connected' ? '3px solid #22c55e' : '3px solid #ef4444' }}>
              <div style={{ display: 'flex', alignItems: 'center' }}>
                <strong style={{ fontSize: 13 }}>{c.player_name}</strong>
                <span style={styles.badge(
                  c.action === 'connected' ? 'rgba(34,197,94,0.15)' : 'rgba(239,68,68,0.15)',
                  c.action === 'connected' ? '#22c55e' : '#ef4444'
                )}>{c.action === 'connected' ? '加入' : '离开'}</span>
                <span style={{ fontSize: 11, color: 'var(--text3)', marginLeft: 'auto' }}>{formatTime(c.logged_at)}</span>
              </div>
              {c.ip_address && <div style={{ fontSize: 11, color: 'var(--text3)', marginTop: 4 }}>IP: {c.ip_address}</div>}
            </div>
          ))}
        </div>
      )}

      {/* Teamkills Feed */}
      {activeTab === 'teamkills' && (
        <div ref={tkRef} style={styles.feedList}>
          {loading && teamkills.length === 0 && <div style={{ textAlign: 'center', padding: 30, color: 'var(--text3)' }}>加载中...</div>}
          {!loading && teamkills.length === 0 && <div style={{ textAlign: 'center', padding: 30, color: 'var(--text3)' }}>暂无误伤事件</div>}
          {teamkills.map(tk => (
            <div key={tk.id} style={{ ...styles.feedItem, borderLeft: '3px solid #ef4444', background: 'rgba(239,68,68,0.05)' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
                <strong style={{ fontSize: 13, color: '#ef6666' }}>{tk.attacker_name}</strong>
                <span style={{ color: 'var(--text3)' }}>→</span>
                <strong style={{ fontSize: 13 }}>{tk.victim_name}</strong>
                <span style={{ fontSize: 11, color: 'var(--text3)', marginLeft: 'auto' }}>{formatTime(tk.logged_at)}</span>
              </div>
              <div style={{ display: 'flex', gap: 16, fontSize: 12, color: 'var(--text2)' }}>
                <span>武器: {tk.weapon}</span>
                <span>伤害: {tk.damage}</span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
