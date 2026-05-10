'use client';

import { useState, useEffect } from 'react';
import { api } from '../lib/api';

interface Server { id: number; server_id: string; name: string; ip: string; rcon_port: number; }

interface Props {
  value: number | null;
  onChange: (serverId: number, server: Server) => void;
  showStatus?: boolean;
}

export default function ServerSwitcher({ value, onChange, showStatus }: Props) {
  const [servers, setServers] = useState<Server[]>([]);
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState('');

  useEffect(() => {
    api('/servers').then(r => r.json()).then(d => setServers(d.data || [])).catch(() => {});
  }, []);

  const filtered = servers.filter(s =>
    !search || s.name.toLowerCase().includes(search.toLowerCase()) || s.ip.includes(search)
  );

  const active = servers.find(s => s.id === value);

  const s = {
    trigger: { display: 'flex', alignItems: 'center', gap: 8, padding: '8px 14px', borderRadius: 8, border: '1px solid var(--border)', background: 'var(--bg2)', color: 'var(--text)', cursor: 'pointer', fontSize: 13, minWidth: 180, justifyContent: 'space-between' as const },
    dropdown: { position: 'absolute' as const, top: '100%', left: 0, marginTop: 4, zIndex: 999, background: 'var(--bg2)', borderRadius: 10, border: '1px solid var(--border)', minWidth: 280, boxShadow: '0 8px 24px rgba(0,0,0,0.4)', overflow: 'hidden' },
    searchBox: { padding: 10, borderBottom: '1px solid var(--border)' },
    searchInput: { width: '100%', padding: '8px 10px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--bg)', color: 'var(--text)', fontSize: 12, boxSizing: 'border-box' as const },
    item: (selected: boolean) => ({ display: 'flex', alignItems: 'center', gap: 10, padding: '10px 14px', cursor: 'pointer', fontSize: 13, color: 'var(--text)', background: selected ? 'rgba(59,130,246,0.1)' : 'transparent', border: 'none', width: '100%', textAlign: 'left' as const }),
    avatar: (name: string) => ({ width: 32, height: 32, borderRadius: 8, background: 'linear-gradient(135deg, #3b82f6, #8b5cf6)', display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#fff', fontSize: 13, fontWeight: 700, flexShrink: 0 }),
  };

  return (
    <div style={{ position: 'relative', display: 'inline-block' }}>
      <button style={s.trigger} onClick={() => setOpen(!open)}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <div style={s.avatar(active?.name || '?')}>{active?.name?.slice(0, 2)?.toUpperCase() || 'SV'}</div>
          <div style={{ textAlign: 'left' }}>
            <div style={{ fontWeight: 600, fontSize: 13 }}>{active?.name || '选择服务器'}</div>
            {active && <div style={{ fontSize: 10, color: 'var(--text3)', fontFamily: 'monospace' }}>{active.ip}:{active.rcon_port}</div>}
          </div>
        </div>
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="m6 9 6 6 6-6"/></svg>
      </button>

      {open && (
        <>
          <div style={{ position: 'fixed', inset: 0, zIndex: 998 }} onClick={() => setOpen(false)} />
          <div style={s.dropdown}>
            <div style={s.searchBox}>
              <input style={s.searchInput} value={search} onChange={e => setSearch(e.target.value)} placeholder="搜索服务器..." autoFocus />
            </div>
            <div style={{ maxHeight: 300, overflow: 'auto' }}>
              {filtered.map(srv => (
                <button key={srv.id} style={s.item(srv.id === value)} onClick={() => { onChange(srv.id, srv); setOpen(false); }}>
                  <div style={s.avatar(srv.name)}>{srv.name.slice(0, 2).toUpperCase()}</div>
                  <div style={{ textAlign: 'left', flex: 1 }}>
                    <div style={{ fontWeight: 500 }}>{srv.name}</div>
                    <div style={{ fontSize: 10, color: 'var(--text3)', fontFamily: 'monospace' }}>{srv.ip}:{srv.rcon_port}</div>
                  </div>
                  {srv.id === value && <span style={{ color: '#3b82f6', fontSize: 11 }}>当前</span>}
                </button>
              ))}
              {filtered.length === 0 && <div style={{ padding: 20, textAlign: 'center', color: 'var(--text3)', fontSize: 13 }}>无匹配服务器</div>}
            </div>
          </div>
        </>
      )}
    </div>
  );
}
