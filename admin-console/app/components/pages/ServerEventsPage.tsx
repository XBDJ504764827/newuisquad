'use client';

import { useState, useEffect, useCallback } from 'react';
import { api } from '../../lib/api';
import { ServerInfo } from '../../types';

type EventTab = 'deployable' | 'tickrate' | 'vehicle' | 'broadcast';

export default function ServerEventsPage() {
  const [servers, setServers] = useState<ServerInfo[]>([]);
  const [sid, setSid] = useState<number | null>(null);
  const [tab, setTab] = useState<EventTab>('deployable');
  const [data, setData] = useState<any[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const perPage = 50;

  useEffect(() => {
    api('/servers').then(r => r.json()).then(d => {
      setServers(d.data || []);
      if (d.data?.length > 0) setSid(d.data[0].id);
    });
  }, []);

  const load = useCallback(async () => {
    if (!sid) return;
    const endpoints: Record<EventTab, string> = {
      deployable: `/servers/${sid}/deployable-events`,
      tickrate: `/servers/${sid}/tick-rate-events`,
      vehicle: `/servers/${sid}/vehicle-events`,
      broadcast: `/servers/${sid}/admin-broadcasts`,
    };
    try {
      const res = await api(`${endpoints[tab]}?page=${page}&per_page=${perPage}`);
      const d = await res.json();
      setData(d.data || []);
      setTotal(d.total || 0);
    } catch {}
  }, [sid, tab, page]);

  useEffect(() => { load(); }, [load]);

  const tabs: { key: EventTab; label: string; icon: string }[] = [
    { key: 'deployable', label: '工事受损', icon: '🏗️' },
    { key: 'tickrate', label: '服务器性能', icon: '📊' },
    { key: 'vehicle', label: '载具记录', icon: '🚁' },
    { key: 'broadcast', label: '管理员广播', icon: '📢' },
  ];

  return (
    <div className="page-view" style={{ padding: 20 }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 16, marginBottom: 16, flexWrap: 'wrap' }}>
        <h2 style={{ margin: 0, fontSize: 20, fontWeight: 600 }}>服务器事件</h2>
        <select value={sid || ''} onChange={e => { setSid(Number(e.target.value)); setPage(1); }}
          style={{ padding: '6px 12px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--bg2)', color: 'var(--text1)' }}>
          {servers.map(s => <option key={s.id} value={s.id}>{s.name}</option>)}
        </select>
      </div>

      {/* Tabs */}
      <div style={{ display: 'flex', gap: 4, marginBottom: 16 }}>
        {tabs.map(t => (
          <button key={t.key} onClick={() => { setTab(t.key); setPage(1); }}
            style={{ padding: '8px 16px', borderRadius: '8px 8px 0 0', border: '1px solid var(--border)', borderBottom: tab === t.key ? '2px solid var(--accent)' : 'none', background: tab === t.key ? 'var(--bg2)' : 'transparent', color: tab === t.key ? 'var(--text1)' : 'var(--text3)', cursor: 'pointer', fontSize: 13, fontWeight: tab === t.key ? 600 : 400 }}>
            {t.icon} {t.label}
          </button>
        ))}
      </div>

      {/* Content */}
      <div style={{ background: 'var(--bg2)', borderRadius: '0 10px 10px 10px', border: '1px solid var(--border)', overflow: 'hidden' }}>
        {tab === 'deployable' && <DeployableTable data={data} />}
        {tab === 'tickrate' && <TickRateTable data={data} />}
        {tab === 'vehicle' && <VehicleTable data={data} />}
        {tab === 'broadcast' && <BroadcastTable data={data} />}
      </div>

      {/* Pagination */}
      {total > perPage && (
        <div style={{ display: 'flex', justifyContent: 'center', gap: 8, marginTop: 16 }}>
          <button disabled={page <= 1} onClick={() => setPage(page - 1)}
            style={pBtn(false)}>上一页</button>
          <span style={{ padding: '6px 12px', fontSize: 13, color: 'var(--text2)' }}>{page} / {Math.ceil(total / perPage)}</span>
          <button disabled={page * perPage >= total} onClick={() => setPage(page + 1)}
            style={pBtn(false)}>下一页</button>
        </div>
      )}
    </div>
  );
}

function DeployableTable({ data }: { data: any[] }) {
  return <table style={tStyle}><thead><tr>{['时间','工事','伤害','武器','攻击者','伤害类型','剩余血量'].map(h => <th key={h} style={th}>{h}</th>)}</tr></thead>
    <tbody>{data.map((r, i) => (
      <tr key={i} style={{ borderBottom: '1px solid var(--border2)' }}>
        <td style={td}>{new Date(r.logged_at).toLocaleString('zh-CN')}</td>
        <td style={td}>{r.deployable}</td>
        <td style={{...td, color: r.damage > 50 ? '#ef4444' : '#f59e0b'}}>{r.damage}</td>
        <td style={td}>{r.weapon}</td>
        <td style={td}>{r.player_suffix}</td>
        <td style={td}>{r.damage_type}</td>
        <td style={td}>{r.health_remaining}</td>
      </tr>))}</tbody></table>;
}

function TickRateTable({ data }: { data: any[] }) {
  return <table style={tStyle}><thead><tr>{['时间','Tick Rate'].map(h => <th key={h} style={th}>{h}</th>)}</tr></thead>
    <tbody>{data.map((r, i) => (
      <tr key={i} style={{ borderBottom: '1px solid var(--border2)' }}>
        <td style={td}>{new Date(r.logged_at).toLocaleString('zh-CN')}</td>
        <td style={{...td, fontWeight: 600, color: r.tick_rate >= 25 ? '#10b981' : r.tick_rate >= 15 ? '#f59e0b' : '#ef4444'}}>{r.tick_rate}</td>
      </tr>))}</tbody></table>;
}

function VehicleTable({ data }: { data: any[] }) {
  return <table style={tStyle}><thead><tr>{['时间','玩家','SteamID','载具','事件'].map(h => <th key={h} style={th}>{h}</th>)}</tr></thead>
    <tbody>{data.map((r, i) => (
      <tr key={i} style={{ borderBottom: '1px solid var(--border2)' }}>
        <td style={td}>{new Date(r.logged_at).toLocaleString('zh-CN')}</td>
        <td style={td}>{r.player_name}</td>
        <td style={{...td, fontFamily: 'monospace', fontSize: 11}}>{r.steam64}</td>
        <td style={td}>{r.vehicle_name}</td>
        <td style={{...td, color: r.event_type === 'enter' ? '#10b981' : '#ef4444'}}>{r.event_type === 'enter' ? '进入' : '离开'}</td>
      </tr>))}</tbody></table>;
}

function BroadcastTable({ data }: { data: any[] }) {
  return <table style={tStyle}><thead><tr>{['时间','管理员','消息'].map(h => <th key={h} style={th}>{h}</th>)}</tr></thead>
    <tbody>{data.map((r, i) => (
      <tr key={i} style={{ borderBottom: '1px solid var(--border2)' }}>
        <td style={td}>{new Date(r.logged_at).toLocaleString('zh-CN')}</td>
        <td style={td}>{r.admin_name}</td>
        <td style={td}>{r.message}</td>
      </tr>))}</tbody></table>;
}

const tStyle: React.CSSProperties = { width: '100%', borderCollapse: 'collapse', fontSize: 13 };
const th: React.CSSProperties = { padding: '8px 12px', textAlign: 'left', fontWeight: 500, color: 'var(--text3)', fontSize: 12, borderBottom: '1px solid var(--border)' };
const td: React.CSSProperties = { padding: '6px 12px', color: 'var(--text2)' };
function pBtn(disabled: boolean): React.CSSProperties { return { padding: '6px 16px', borderRadius: 6, border: '1px solid var(--border)', background: disabled ? 'var(--bg3)' : 'var(--bg2)', color: disabled ? 'var(--text3)' : 'var(--text1)', cursor: disabled ? 'default' : 'pointer', fontSize: 13 }; }
