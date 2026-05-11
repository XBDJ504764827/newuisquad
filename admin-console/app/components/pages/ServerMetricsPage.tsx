'use client';

import { useState, useEffect, useCallback } from 'react';
import { api } from '../../lib/api';

interface Server { id: number; server_id: string; name: string; ip: string; rcon_port: number; }
interface ServerHealthItem { server_id: number; rcon_healthy: boolean; agent_connected: boolean; status: 'online' | 'degraded' | 'offline' | 'unknown'; last_check: string; player_count: number | null; map_name: string | null; }
interface EnhancedServer { id: number; name: string; ip: string; rcon_port: number; health: ServerHealthItem; stats_24h: any; }

type Period = '1h' | '6h' | '24h' | '7d' | '30d';

export default function ServerMetricsPage() {
  const [servers, setServers] = useState<Server[]>([]);
  const [enhancedServers, setEnhancedServers] = useState<EnhancedServer[]>([]);
  const [serverId, setServerId] = useState<number | null>(null);
  const [activeTab, setActiveTab] = useState<'overview' | 'detail'>('overview');
  const [period, setPeriod] = useState<Period>('24h');
  const [loading, setLoading] = useState(false);
  const [serverInfo, setServerInfo] = useState<any>(null);

  useEffect(() => {
    api('/servers').then(r => r.json()).then(d => {
      const list = d.data || [];
      setServers(list);
      if (list.length > 0) setServerId(list[0].id);
    }).catch(() => {});
  }, []);

  const loadOverview = useCallback(async () => {
    setLoading(true);
    try {
      const res = await api('/servers/enhanced');
      if (res.ok) { const d = await res.json(); setEnhancedServers(d.data || []); }
    } catch {}
    setLoading(false);
  }, []);

  const loadDetail = useCallback(async () => {
    if (!serverId) return;
    setLoading(true);
    try {
      const [statsRes, enhancedRes] = await Promise.all([
        api(`/servers/${serverId}/stats`),
        api('/servers/enhanced'),
      ]);
      const statsData = await statsRes.json();
      const enhancedData = await enhancedRes.json();
      const enhanced = (enhancedData.data || []).find((s: any) => s.id === serverId);
      setServerInfo({
        name: enhanced?.name || '', ip: enhanced?.ip || '',
        health: enhanced?.health?.status || 'unknown',
        player_count: enhanced?.health?.player_count || 0,
        map_name: enhanced?.health?.map_name || '',
        stats_24h: enhanced?.stats_24h || statsData || {},
      });
    } catch {}
    setLoading(false);
  }, [serverId]);

  useEffect(() => { if (activeTab === 'overview') loadOverview(); else loadDetail(); }, [activeTab, loadOverview, loadDetail]);

  const statusColor = (s: string) => s === 'online' ? '#10b981' : s === 'degraded' ? '#f59e0b' : s === 'unknown' ? '#6b7280' : '#ef4444';
  const statusLabel = (s: string) => s === 'online' ? '在线' : s === 'degraded' ? '降级' : s === 'unknown' ? '未知' : '离线';
  const periodLabel: Record<Period, string> = { '1h': '近1小时', '6h': '近6小时', '24h': '近24小时', '7d': '近7天', '30d': '近30天' };

  const online = enhancedServers.filter(s => s.health.status === 'online').length;
  const degraded = enhancedServers.filter(s => s.health.status === 'degraded').length;
  const offline = enhancedServers.filter(s => s.health.status === 'offline').length;

  const styles = {
    tabs: { display: 'flex', gap: 2, marginBottom: 20, borderBottom: '1px solid var(--border)' },
    tab: (active: boolean) => ({ padding: '10px 20px', border: 'none', background: active ? 'var(--text)' : 'transparent', color: active ? 'var(--bg)' : 'var(--text2)', cursor: 'pointer', borderRadius: '8px 8px 0 0', fontSize: 13, fontWeight: active ? 600 : 400 }),
    card: { background: 'var(--bg2)', borderRadius: 10, border: '1px solid var(--border)', padding: 16 },
    statCard: { background: 'var(--bg2)', borderRadius: 10, border: '1px solid var(--border)', padding: 16, textAlign: 'center' as const, minWidth: 100, flex: 1 },
    btn: { padding: '6px 14px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--bg2)', color: 'var(--text)', cursor: 'pointer', fontSize: 12 },
    select: { padding: '6px 12px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--bg2)', color: 'var(--text)', fontSize: 13, minWidth: 200 },
  };

  const demoData = Array.from({ length: 24 }, () => Math.floor(Math.random() * 40 + 20));

  function MiniChart({ data, color, h = 50 }: { data: number[]; color: string; h?: number }) {
    const max = Math.max(...data, 1); const min = Math.min(...data, 0); const range = max - min || 1;
    const w = 200;
    const pts = data.map((v, i) => `${(i / (data.length - 1 || 1)) * w},${h - ((v - min) / range) * (h - 4) - 2}`).join(' ');
    return (
      <svg width={w} height={h}><polyline points={pts} fill="none" stroke={color} strokeWidth="2" />
        {data.map((v, i) => <circle key={i} cx={(i / (data.length - 1 || 1)) * w} cy={h - ((v - min) / range) * (h - 4) - 2} r="1.5" fill={color} />)}</svg>
    );
  }

  return (
    <div style={{ padding: 20 }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 16, flexWrap: 'wrap' }}>
        <h2 style={{ margin: 0, fontSize: 20, fontWeight: 600 }}>服务器状态</h2>
        {activeTab === 'detail' && (
          <select value={serverId || ''} onChange={e => setServerId(Number(e.target.value))} style={styles.select}>
            {servers.map(s => <option key={s.id} value={s.id}>{s.name}</option>)}
          </select>
        )}
        <button onClick={() => activeTab === 'overview' ? loadOverview() : loadDetail()} style={{ ...styles.btn, background: 'var(--text)', color: 'var(--bg)' }}>刷新</button>
      </div>

      <div style={styles.tabs}>
        <button onClick={() => setActiveTab('overview')} style={styles.tab(activeTab === 'overview')}>全服概览</button>
        <button onClick={() => setActiveTab('detail')} style={styles.tab(activeTab === 'detail')}>详细指标</button>
      </div>

      {loading && <div style={{ textAlign: 'center', padding: 40, color: 'var(--text3)' }}>加载中...</div>}

      {/* ===== Overview Tab ===== */}
      {activeTab === 'overview' && !loading && (
        <>
          <div style={{ display: 'flex', gap: 12, marginBottom: 16 }}>
            <div style={styles.statCard}><div style={{ fontSize: 28, fontWeight: 700 }}>{enhancedServers.length}</div><div style={{ fontSize: 11, color: 'var(--text3)' }}>总数</div></div>
            <div style={styles.statCard}><div style={{ fontSize: 28, fontWeight: 700, color: '#10b981' }}>{online}</div><div style={{ fontSize: 11, color: 'var(--text3)' }}>在线</div></div>
            <div style={styles.statCard}><div style={{ fontSize: 28, fontWeight: 700, color: '#f59e0b' }}>{degraded}</div><div style={{ fontSize: 11, color: 'var(--text3)' }}>降级</div></div>
            <div style={styles.statCard}><div style={{ fontSize: 28, fontWeight: 700, color: '#ef4444' }}>{offline}</div><div style={{ fontSize: 11, color: 'var(--text3)' }}>离线</div></div>
          </div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(320px, 1fr))', gap: 12 }}>
            {enhancedServers.map(s => (
              <div key={s.id} style={{ ...styles.card, cursor: 'pointer' }} onClick={() => { setServerId(s.id); setActiveTab('detail'); }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
                  <div>
                    <div style={{ fontSize: 15, fontWeight: 600, color: 'var(--text)' }}>{s.name}</div>
                    <div style={{ fontSize: 11, color: 'var(--text3)', fontFamily: 'monospace' }}>{s.ip}:{s.rcon_port}</div>
                  </div>
                  <span style={{ fontSize: 12, fontWeight: 600, color: statusColor(s.health.status) }}>
                    <span style={{ width: 8, height: 8, borderRadius: 4, background: statusColor(s.health.status), display: 'inline-block', marginRight: 6 }} />
                    {statusLabel(s.health.status)}
                  </span>
                </div>
                <div style={{ display: 'flex', gap: 16, fontSize: 12, color: 'var(--text2)' }}>
                  <span>玩家: {s.health.player_count ?? '-'}</span>
                  <span>地图: {s.health.map_name || '-'}</span>
                  {s.stats_24h && <span>24h击杀: {s.stats_24h.kill_count || 0}</span>}
                </div>
              </div>
            ))}
            {enhancedServers.length === 0 && <div style={{ textAlign: 'center', color: 'var(--text3)', gridColumn: '1 / -1', padding: 30 }}>暂无服务器数据</div>}
          </div>
        </>
      )}

      {/* ===== Detail Tab ===== */}
      {activeTab === 'detail' && !loading && serverInfo && (
        <>
          <div style={{ display: 'flex', gap: 12, marginBottom: 16, alignItems: 'center', flexWrap: 'wrap' }}>
            <span style={{ width: 10, height: 10, borderRadius: 5, background: statusColor(serverInfo.health) }} />
            <span style={{ fontSize: 14, fontWeight: 600 }}>{serverInfo.name}</span>
            <span style={{ fontSize: 12, color: 'var(--text3)', fontFamily: 'monospace' }}>{serverInfo.ip}</span>
            <span style={{ fontSize: 13, color: 'var(--text2)' }}>玩家: {serverInfo.player_count || '-'}</span>
            <span style={{ fontSize: 13, color: 'var(--text2)' }}>地图: {serverInfo.map_name || '-'}</span>
            <div style={{ marginLeft: 'auto', display: 'flex', gap: 4 }}>
              {(Object.keys(periodLabel) as Period[]).map(p => (
                <button key={p} onClick={() => setPeriod(p)} style={{ ...styles.btn, background: period === p ? 'var(--text)' : 'var(--bg2)', color: period === p ? 'var(--bg)' : 'var(--text)' }}>{periodLabel[p]}</button>
              ))}
            </div>
          </div>

          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(180px, 1fr))', gap: 12, marginBottom: 16 }}>
            {[{ icon: '👥', label: '峰值玩家', value: serverInfo.stats_24h?.player_count || serverInfo.player_count, color: '#3b82f6' },
              { icon: '💀', label: '24h击杀', value: serverInfo.stats_24h?.kill_count || 0, color: '#ef4444' },
              { icon: '☠️', label: '24h误伤', value: serverInfo.stats_24h?.teamkill_count || 0, color: '#f59e0b' },
              { icon: '⚔️', label: '24h比赛', value: serverInfo.stats_24h?.match_count || 0, color: '#8b5cf6' },
              { icon: '💬', label: '24h聊天', value: serverInfo.stats_24h?.chat_count || 0, color: '#10b981' },
              { icon: '⚠', label: '24h错误', value: serverInfo.stats_24h?.error_count || 0, color: '#ec4899' },
            ].map(m => (
              <div key={m.label} style={{ ...styles.card, display: 'flex', alignItems: 'center', gap: 12 }}>
                <span style={{ fontSize: 24 }}>{m.icon}</span>
                <div><div style={{ fontSize: 22, fontWeight: 700, color: m.color }}>{typeof m.value === 'number' ? m.value.toLocaleString() : m.value}</div><div style={{ fontSize: 11, color: 'var(--text3)' }}>{m.label}</div></div>
              </div>
            ))}
          </div>

          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(220px, 1fr))', gap: 12 }}>
            <div style={{ ...styles.card }}>
              <div style={{ fontSize: 13, fontWeight: 600, marginBottom: 8, color: 'var(--text2)' }}>玩家趋势</div>
              <MiniChart data={demoData} color="#3b82f6" h={60} />
            </div>
            <div style={{ ...styles.card }}>
              <div style={{ fontSize: 13, fontWeight: 600, marginBottom: 8, color: 'var(--text2)' }}>击杀趋势</div>
              <MiniChart data={demoData.map(v => Math.floor(v * 0.6))} color="#ef4444" h={60} />
            </div>
            <div style={{ ...styles.card }}>
              <div style={{ fontSize: 13, fontWeight: 600, marginBottom: 8, color: 'var(--text2)' }}>误伤趋势</div>
              <MiniChart data={demoData.map(v => Math.floor(v * 0.05))} color="#f59e0b" h={60} />
            </div>
          </div>
        </>
      )}
    </div>
  );
}
