'use client';

import { useState, useEffect, useCallback } from 'react';
import { api } from '../../lib/api';

interface PlayerIdentity {
  canonical_id: string;
  primary_steam_id: string;
  primary_eos_id: string;
  primary_name: string;
  all_steam_ids: string[];
  all_eos_ids: string[];
  all_names: string[];
  total_sessions: number;
  first_seen: string | null;
  last_seen: string | null;
}

function timeAgo(ts: string | null): string {
  if (!ts) return '未知';
  const d = new Date(ts);
  const now = new Date();
  const diff = now.getTime() - d.getTime();
  const mins = Math.floor(diff / 60000);
  const hrs = Math.floor(mins / 60);
  const days = Math.floor(hrs / 24);
  if (days > 0) return `${days} 天前`;
  if (hrs > 0) return `${hrs} 小时前`;
  if (mins > 0) return `${mins} 分钟前`;
  return '刚刚';
}

export default function IdentityAltsPage() {
  const [identities, setIdentities] = useState<PlayerIdentity[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [computing, setComputing] = useState(false);
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const perPage = 30;

  const load = useCallback(async (p: number) => {
    setLoading(true);
    setError(null);
    try {
      const res = await api(`/identities?page=${p}&per_page=${perPage}`);
      if (res.status === 403) { setError('permission_denied'); setLoading(false); return; }
      const data = await res.json();
      if (data.error) { setError(data.error); setLoading(false); return; }
      setIdentities(data.data || []);
      setTotal(data.total || 0);
    } catch (e: any) { setError(e.message); }
    setLoading(false);
  }, []);

  useEffect(() => { load(page); }, [page, load]);

  const handleCompute = async () => {
    setComputing(true);
    try {
      const res = await api('/identity/compute', { method: 'POST' });
      const data = await res.json();
      if (data.success) {
        setError(null);
        load(page);
      } else {
        setError(data.error || '计算失败');
      }
    } catch (e: any) { setError(e.message); }
    setComputing(false);
  };

  const toggleExpand = (id: string) => {
    const next = new Set(expanded);
    next.has(id) ? next.delete(id) : next.add(id);
    setExpanded(next);
  };

  const navigateToPlayer = (steamId: string) => {
    if (steamId) window.location.hash = `player-detail?steam_id=${steamId}`;
  };

  const totalPages = Math.ceil(total / perPage);
  const multiAccountGroups = identities.filter(g => g.all_steam_ids.length > 1 || g.all_eos_ids.length > 1);

  const styles = {
    container: { padding: 20, maxWidth: 1100, margin: '0 auto' },
    header: { display: 'flex', alignItems: 'center', gap: 12, marginBottom: 20, flexWrap: 'wrap' as const },
    btn: { padding: '8px 16px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--text)', color: 'var(--bg)', cursor: 'pointer', fontWeight: 500, fontSize: 13 },
    btnOutline: { padding: '6px 14px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--bg2)', color: 'var(--text)', cursor: 'pointer', fontSize: 12 },
    card: { background: 'var(--bg2)', borderRadius: 10, border: '1px solid var(--border)', marginBottom: 12, overflow: 'hidden' },
    groupHeader: { padding: '14px 18px', cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap' as const, gap: 8 },
    playerRow: { padding: '10px 18px', display: 'flex', alignItems: 'center', justifyContent: 'space-between', borderTop: '1px solid var(--border)', cursor: 'pointer', fontSize: 13 },
    badge: (bg: string, color: string) => ({ display: 'inline-block', padding: '2px 10px', borderRadius: 10, fontSize: 11, fontWeight: 600, background: bg, color }),
    statCard: { background: 'var(--bg2)', borderRadius: 10, border: '1px solid var(--border)', padding: 16, textAlign: 'center' as const, minWidth: 100, flex: 1 },
    pagination: { display: 'flex', justifyContent: 'center', gap: 8, marginTop: 20, alignItems: 'center' },
    pageBtn: (active: boolean) => ({ padding: '6px 12px', borderRadius: 6, border: '1px solid var(--border)', background: active ? 'var(--text)' : 'var(--bg2)', color: active ? 'var(--bg)' : 'var(--text)', cursor: 'pointer', fontSize: 12, fontWeight: active ? 600 : 400 }),
  };

  if (error === 'permission_denied') {
    return (
      <div style={styles.container}>
        <div style={{ textAlign: 'center', padding: 60, color: 'var(--text3)' }}>
          <div style={{ fontSize: 48, marginBottom: 16 }}>🔒</div>
          <p style={{ fontSize: 16, marginBottom: 8 }}>需要超级管理员权限</p>
          <p style={{ fontSize: 13 }}>此功能仅限超级管理员使用</p>
        </div>
      </div>
    );
  }

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h2 style={{ margin: 0, fontSize: 20, fontWeight: 600 }}>小号关联检测</h2>
        <button onClick={() => load(page)} style={{ ...styles.btn, background: 'var(--bg2)', color: 'var(--text)' }}>刷新</button>
        <button onClick={handleCompute} disabled={computing} style={{ ...styles.btn, opacity: computing ? 0.6 : 1 }}>
          {computing ? '计算中...' : '重新计算关联'}
        </button>
        <span style={{ fontSize: 12, color: 'var(--text3)', marginLeft: 'auto' }}>通过共享标识符检测关联账号</span>
      </div>

      {/* Stats bar */}
      <div style={{ display: 'flex', gap: 12, marginBottom: 20, flexWrap: 'wrap' }}>
        <div style={styles.statCard}>
          <div style={{ fontSize: 24, fontWeight: 700, color: '#3b82f6' }}>{total}</div>
          <div style={{ fontSize: 11, color: 'var(--text3)' }}>身份分组总数</div>
        </div>
        <div style={styles.statCard}>
          <div style={{ fontSize: 24, fontWeight: 700, color: '#f59e0b' }}>{multiAccountGroups.length}</div>
          <div style={{ fontSize: 11, color: 'var(--text3)' }}>疑似多账号分组</div>
        </div>
        <div style={styles.statCard}>
          <div style={{ fontSize: 24, fontWeight: 700, color: '#ef4444' }}>
            {identities.reduce((sum, g) => sum + g.all_steam_ids.length, 0)}
          </div>
          <div style={{ fontSize: 11, color: 'var(--text3)' }}>关联Steam ID总数</div>
        </div>
        <div style={styles.statCard}>
          <div style={{ fontSize: 24, fontWeight: 700, color: '#8b5cf6' }}>
            {identities.filter(g => g.all_steam_ids.length > 2).length}
          </div>
          <div style={{ fontSize: 11, color: 'var(--text3)' }}>≥3个关联账号</div>
        </div>
      </div>

      {error && error !== 'permission_denied' && (
        <div style={{ padding: '10px 14px', background: 'rgba(239,68,68,0.1)', color: '#ef4444', borderRadius: 6, marginBottom: 12, fontSize: 13 }}>{error}</div>
      )}

      {loading && <div style={{ textAlign: 'center', padding: 40, color: 'var(--text3)' }}>加载中...</div>}

      {!loading && identities.length === 0 && (
        <div style={{ textAlign: 'center', padding: 40, color: 'var(--text3)' }}>
          <p style={{ fontSize: 16, marginBottom: 8 }}>暂无身份关联数据</p>
          <p style={{ fontSize: 13 }}>点击"重新计算关联"来检测小号</p>
        </div>
      )}

      {identities.map(group => {
        const isExpanded = expanded.has(group.canonical_id);
        // Build player list for this identity group
        const players: { steam_id: string; eos_id: string; name: string }[] = [];
        const steamSet = new Set(group.all_steam_ids);
        const eosSet = new Set(group.all_eos_ids);
        const maxLen = Math.max(group.all_steam_ids.length, group.all_eos_ids.length, group.all_names.length);
        for (let i = 0; i < maxLen; i++) {
          players.push({
            steam_id: group.all_steam_ids[i] || '',
            eos_id: group.all_eos_ids[i] || '',
            name: group.all_names[i] || '',
          });
        }

        return (
          <div key={group.canonical_id} style={styles.card}>
            <div style={styles.groupHeader} onClick={() => toggleExpand(group.canonical_id)}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 10, flex: 1, minWidth: 0 }}>
                <div style={{ display: 'flex', marginRight: 4 }}>
                  {players.slice(0, 3).map((p, idx) => (
                    <div key={idx} style={{
                      width: 28, height: 28, borderRadius: 14, background: 'var(--bg)', border: '2px solid var(--border)',
                      display: 'flex', alignItems: 'center', justifyContent: 'center',
                      fontSize: 12, fontWeight: 700, color: 'var(--text)', marginLeft: idx > 0 ? -8 : 0,
                    }}>
                      {(p.name || '?')[0].toUpperCase()}
                    </div>
                  ))}
                  {players.length > 3 && (
                    <div style={{
                      width: 28, height: 28, borderRadius: 14, background: 'var(--bg)', border: '2px solid var(--border)',
                      display: 'flex', alignItems: 'center', justifyContent: 'center',
                      fontSize: 10, color: 'var(--text3)', marginLeft: -8,
                    }}>
                      +{players.length - 3}
                    </div>
                  )}
                </div>
                <div style={{ minWidth: 0 }}>
                  <div style={{ fontSize: 14, fontWeight: 600, color: 'var(--text)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                    {group.primary_name || '未知玩家'}
                  </div>
                  <div style={{ fontSize: 11, color: 'var(--text3)' }}>
                    {group.all_steam_ids.length} Steam + {group.all_eos_ids.length} EOS · {timeAgo(group.last_seen)}
                  </div>
                </div>
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: 8, flexShrink: 0 }}>
                {group.all_steam_ids.length > 1 && (
                  <span style={styles.badge('rgba(245,158,11,0.15)', '#f59e0b')}>疑似小号</span>
                )}
                {group.all_steam_ids.length >= 3 && (
                  <span style={styles.badge('rgba(239,68,68,0.15)', '#ef4444')}>多账号</span>
                )}
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"
                  style={{ transform: isExpanded ? 'rotate(180deg)' : 'rotate(0deg)', transition: 'transform 0.2s' }}>
                  <path d="m6 9 6 6 6-6"/>
                </svg>
              </div>
            </div>
            {isExpanded && (
              <div>
                {players.map((p, idx) => (
                  <div key={idx} style={styles.playerRow} onClick={() => navigateToPlayer(p.steam_id)}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 8, flex: 1, minWidth: 0 }}>
                      <div style={{
                        width: 24, height: 24, borderRadius: 12, background: 'var(--bg)', border: '1px solid var(--border)',
                        display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: 10, fontWeight: 700, color: 'var(--text2)',
                      }}>
                        {(p.name || '?')[0].toUpperCase()}
                      </div>
                      <span style={{ fontWeight: 500, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{p.name || '无名玩家'}</span>
                    </div>
                    <div style={{ display: 'flex', gap: 12, alignItems: 'center', flexShrink: 0 }}>
                      {p.steam_id && <code style={{ fontSize: 10, color: 'var(--text3)', background: 'var(--bg)', padding: '2px 6px', borderRadius: 4 }}>{p.steam_id}</code>}
                      {p.eos_id && <code style={{ fontSize: 10, color: 'var(--text3)', background: 'var(--bg)', padding: '2px 6px', borderRadius: 4 }}>{p.eos_id}</code>}
                    </div>
                  </div>
                ))}
                <div style={{ padding: '10px 18px', display: 'flex', gap: 12, flexWrap: 'wrap', borderTop: '1px solid var(--border)' }}>
                  {group.all_names.slice(0, 10).map((name, idx) => (
                    <span key={idx} style={{ fontSize: 10, padding: '2px 8px', borderRadius: 4, background: 'rgba(139,92,246,0.1)', color: '#8b5cf6' }}>{name}</span>
                  ))}
                  {group.all_names.length > 10 && <span style={{ fontSize: 10, color: 'var(--text3)' }}>+{group.all_names.length - 10} 更多</span>}
                </div>
              </div>
            )}
          </div>
        );
      })}

      {totalPages > 1 && (
        <div style={styles.pagination}>
          <button onClick={() => setPage(p => Math.max(1, p - 1))} disabled={page <= 1} style={{ ...styles.pageBtn(false), opacity: page <= 1 ? 0.4 : 1 }}>上一页</button>
          {Array.from({ length: Math.min(totalPages, 7) }, (_, i) => {
            const start = Math.max(1, Math.min(page - 3, totalPages - 6));
            const pn = start + i;
            if (pn > totalPages) return null;
            return <button key={pn} onClick={() => setPage(pn)} style={styles.pageBtn(pn === page)}>{pn}</button>;
          })}
          <button onClick={() => setPage(p => Math.min(totalPages, p + 1))} disabled={page >= totalPages} style={{ ...styles.pageBtn(false), opacity: page >= totalPages ? 0.4 : 1 }}>下一页</button>
        </div>
      )}
    </div>
  );
}
