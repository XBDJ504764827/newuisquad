'use client';

import { useState, useEffect, useCallback } from 'react';
import { api } from '../../lib/api';

// ─── 类型 ───────────────────────────────────────────
interface PlayerProfile {
  steam_id: string; eos_id: string; player_name: string;
  last_known_ip?: string; can_view_ip?: boolean;
  last_seen: string | null; first_seen: string | null;
  total_play_time: number; total_sessions: number;
  statistics: PlayerStats;
  current_match_statistics: PlayerStats;
  chat_history: ChatEntry[];
  combat_history: CombatEntry[];
  violations: ViolationEntry[];
  violation_summary: ViolationSummary;
  teamkill_metrics: TeamkillMetrics;
  weapon_stats: WeaponStat[];
  name_history: NameHistoryEntry[];
  active_bans: ActiveBan[];
  risk_indicators: RiskIndicator[];
  identity?: IdentityInfo | null;
}

interface PlayerStats {
  kills: number; deaths: number; teamkills: number;
  revives: number; times_revived: number;
  damage_dealt: number; damage_taken: number; kd_ratio: number;
}

interface ChatEntry { player_name: string; message: string; channel: string; logged_at: string; }
interface CombatEntry { event_time: string; event_type: string; weapon: string; damage: number; teamkill: boolean; other_name: string; other_steam64: string; is_attacker: boolean; }
interface ViolationEntry { player_name: string; message: string; category: string; action_taken: string; logged_at: string; }
interface ViolationSummary { total_warns: number; total_kicks: number; total_bans: number; }
interface TeamkillMetrics { total_teamkills: number; teamkills_per_session: number; teamkill_ratio: number; recent_teamkills: number; }
interface WeaponStat { weapon: string; kills: number; teamkills: number; }
interface NameHistoryEntry { name: string; session_count: number; }
interface ActiveBan { ban_id: string; server_name: string; reason: string; permanent: boolean; expires_at?: string; created_at: string; admin_name: string; }
interface RiskIndicator { type: string; severity: 'critical' | 'high' | 'medium' | 'low'; description: string; }
interface IdentityInfo { canonical_id: string; primary_name: string; all_steam_ids: string[]; all_eos_ids: string[]; all_names: string[]; total_sessions: number; identity_status: string; }

// ─── 工具函数 ──────────────────────────────────────
function formatDate(ts: string | null): string {
  if (!ts) return '-';
  return new Date(ts).toLocaleString('zh-CN');
}

function timeAgo(ts: string | null): string {
  if (!ts) return '-';
  const diff = Date.now() - new Date(ts).getTime();
  const s = Math.floor(diff / 1000);
  if (s < 60) return '刚刚';
  const m = Math.floor(s / 60); if (m < 60) return `${m}分钟前`;
  const h = Math.floor(m / 60); if (h < 24) return `${h}小时前`;
  const d = Math.floor(h / 24); if (d < 30) return `${d}天前`;
  const mo = Math.floor(d / 30); if (mo < 12) return `${mo}个月前`;
  return `${Math.floor(mo / 12)}年前`;
}

// ─── 主组件 ─────────────────────────────────────────
export default function PlayerDetailPage() {
  const [steam64, setSteam64] = useState<string>('');
  const [profile, setProfile] = useState<PlayerProfile | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState('chat');
  const [copied, setCopied] = useState<string | null>(null);

  // 从 URL hash 解析参数（含监听以支持同页面切换）
  const parseHash = useCallback(() => {
    const hash = window.location.hash.replace('#', '');
    const [page, qs] = hash.split('?');
    if (page === 'player-detail' && qs) {
      const params = new URLSearchParams(qs);
      const sid = params.get('steam64') || '';
      if (sid && sid !== steam64) {
        setSteam64(sid);
        setActiveTab('chat');
      }
    }
  }, [steam64]);

  useEffect(() => {
    parseHash();
    window.addEventListener('hashchange', parseHash);
    return () => window.removeEventListener('hashchange', parseHash);
  }, [parseHash]);

  useEffect(() => {
    if (!steam64) return;
    setLoading(true);
    setError(null);
    api(`/player-profile/${steam64}`)
      .then(r => r.json())
      .then(d => {
        if (d.error) { setError(d.error); return; }
        setProfile(d.data?.player || null);
      })
      .catch(e => setError(e.message))
      .finally(() => setLoading(false));
  }, [steam64]);

  const copyText = useCallback((text: string, field: string) => {
    navigator.clipboard.writeText(text);
    setCopied(field);
    setTimeout(() => setCopied(null), 2000);
  }, []);

  function goBack() {
    window.location.hash = 'player-info';
  }

  // ─── 加载状态 ────────────────────────────────────
  if (loading) {
    return <div className="page-view" style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '60vh' }}>
      <div style={{ textAlign: 'center' }}>
        <div style={{ width: 40, height: 40, border: '3px solid var(--border)', borderTopColor: 'var(--primary)', borderRadius: '50%', animation: 'spin 0.8s linear infinite', margin: '0 auto 16px' }} />
        <p style={{ color: 'var(--text3)', fontSize: 14 }}>加载玩家档案...</p>
      </div>
    </div>;
  }

  if (error) {
    return <div className="page-view" style={{ padding: 24 }}>
      <button className="rcon-btn" style={{ width: 'auto', padding: '6px 14px', marginBottom: 16 }} onClick={goBack}>&larr; 返回</button>
      <div className="card" style={{ padding: 24, textAlign: 'center', color: 'var(--danger)' }}>{error}</div>
    </div>;
  }

  if (!profile) return null;

  // ─── 渲染 ─────────────────────────────────────────
  const totalViolations = profile.violation_summary.total_warns + profile.violation_summary.total_kicks + profile.violation_summary.total_bans;
  const tkPct = (profile.teamkill_metrics.teamkill_ratio * 100).toFixed(1);
  const hasLinked = profile.identity && (profile.identity.all_steam_ids?.length > 1 || profile.identity.all_eos_ids?.length > 1);
  const tabs = [
    { id: 'chat', label: '聊天记录' },
    { id: 'combat', label: '战斗记录' },
    { id: 'violations', label: '违规记录' },
    { id: 'teamkills', label: '误伤分析' },
    { id: 'weapons', label: '武器统计' },
    { id: 'names', label: '名称历史' },
  ];

  return (
    <div className="page-view" style={{ gap: 16 }}>
      {/* 返回按钮 */}
      <button className="rcon-btn" style={{ width: 'auto', padding: '6px 14px', alignSelf: 'flex-start' }} onClick={goBack}>
        &larr; 返回玩家列表
      </button>

      {/* ── 顶部玩家信息卡片 ── */}
      <div className="card">
        <div className="card-header">
          <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
            <div className="card-title" style={{ fontSize: 20 }}>{profile.player_name}</div>
            {profile.name_history.length > 1 && (
              <span style={{ fontSize: 12, color: 'var(--text3)', background: 'var(--bg3)', padding: '2px 8px', borderRadius: 10 }}>
                {profile.name_history.length} 个别名
              </span>
            )}
          </div>
        </div>
        <div className="card-body" style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
          {/* 关联身份提示 */}
          {hasLinked && (
            <div style={{ padding: '8px 12px', borderRadius: 6, background: 'rgba(59,130,246,0.1)', border: '1px solid rgba(59,130,246,0.2)', fontSize: 13, color: 'var(--text2)' }}>
              检测到关联身份，统计已合并展示。
              {profile.identity!.all_steam_ids.length > 1 && (
                <div style={{ marginTop: 4, display: 'flex', gap: 4, flexWrap: 'wrap' }}>
                  {profile.identity!.all_steam_ids.map((id: string) => (
                    <code key={id} style={{ fontSize: 11, background: 'var(--bg3)', padding: '1px 6px', borderRadius: 3 }}>{id}</code>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* ID 行 */}
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: 12 }}>
            {[
              { label: 'Steam64', value: profile.steam_id, field: 'steam' },
              { label: 'EOS ID', value: profile.eos_id || '-', field: 'eos' },
              ...(profile.can_view_ip ? [{ label: '最近 IP', value: profile.last_known_ip || '-', field: 'ip' }] : []),
            ].map(({ label, value, field }) => (
              <div key={field}>
                <div style={{ fontSize: 11, color: 'var(--text3)', marginBottom: 2 }}>{label}</div>
                <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                  <code style={{ fontSize: 12, background: 'var(--bg3)', padding: '2px 8px', borderRadius: 4, flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{value}</code>
                  <button className="rcon-btn" style={{ width: 'auto', padding: '2px 8px', fontSize: 11 }}
                    onClick={() => copyText(value, field)}>
                    {copied === field ? '✓' : '复制'}
                  </button>
                </div>
              </div>
            ))}
          </div>

          {/* 时间统计 */}
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: 16, paddingTop: 8 }}>
            {[
              { label: '首次出现', value: formatDate(profile.first_seen) },
              { label: '最近出现', value: `${timeAgo(profile.last_seen)} (${formatDate(profile.last_seen)})` },
              { label: '服务器', value: `${profile.total_sessions} 台` },
              { label: '场次', value: profile.total_sessions },
            ].map(({ label, value }) => (
              <div key={label}>
                <div style={{ fontSize: 11, color: 'var(--text3)' }}>{label}</div>
                <div style={{ fontSize: 14, fontWeight: 600 }}>{value}</div>
              </div>
            ))}
          </div>

          {/* 外部链接 */}
          <div style={{ borderTop: '1px solid var(--border)', paddingTop: 10, display: 'flex', gap: 8 }}>
            <a href={`https://steamcommunity.com/profiles/${profile.steam_id}`} target="_blank" rel="noopener noreferrer"
              className="rcon-btn" style={{ width: 'auto', padding: '4px 12px', fontSize: 12, textDecoration: 'none', background: '#171a21', color: '#fff' }}>Steam</a>
            <a href={`https://www.battlemetrics.com/players?filter[search]=${profile.player_name}`} target="_blank" rel="noopener noreferrer"
              className="rcon-btn" style={{ width: 'auto', padding: '4px 12px', fontSize: 12, textDecoration: 'none', background: '#f26a21', color: '#fff' }}>BattleMetrics</a>
            <a href={`https://communitybanlist.com/search/${profile.steam_id}`} target="_blank" rel="noopener noreferrer"
              className="rcon-btn" style={{ width: 'auto', padding: '4px 12px', fontSize: 12, textDecoration: 'none', background: '#7c3aed', color: '#fff' }}>CBL</a>
          </div>
        </div>
      </div>

      {/* ── 风险指标 ── */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))', gap: 12 }}>
        {[
          { label: 'TK 比例', value: `${tkPct}%`, color: parseFloat(tkPct) > 10 ? 'var(--danger)' : parseFloat(tkPct) > 5 ? '#f97316' : 'var(--green)', sub: `总计 ${profile.teamkill_metrics.total_teamkills} 次` },
          { label: '近期 TK', value: profile.teamkill_metrics.recent_teamkills, color: profile.teamkill_metrics.recent_teamkills >= 5 ? 'var(--danger)' : profile.teamkill_metrics.recent_teamkills >= 3 ? '#f97316' : 'var(--text)', sub: '近7天' },
          { label: '违规次数', value: totalViolations, color: totalViolations >= 10 ? 'var(--danger)' : totalViolations >= 5 ? '#f97316' : 'var(--text)', sub: `W${profile.violation_summary.total_warns} K${profile.violation_summary.total_kicks} B${profile.violation_summary.total_bans}` },
          { label: '使用名称', value: profile.name_history.length, color: profile.name_history.length > 5 ? '#f97316' : 'var(--text)', sub: '个别名' },
        ].map(({ label, value, color, sub }) => (
          <div key={label} className="card" style={{ padding: '12px 16px', textAlign: 'center' }}>
            <div style={{ fontSize: 11, color: 'var(--text3)', marginBottom: 4 }}>{label}</div>
            <div style={{ fontSize: 28, fontWeight: 700, color }}>{value}</div>
            <div style={{ fontSize: 11, color: 'var(--text3)' }}>{sub}</div>
          </div>
        ))}
      </div>

      {/* ── K/D 统计网格 ── */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(120px, 1fr))', gap: 12 }}>
        {[
          { label: 'K/D 比', value: profile.statistics.kd_ratio.toFixed(2), sub: `${profile.statistics.kills} / ${profile.statistics.deaths}` },
          { label: '击杀', value: profile.statistics.kills, color: 'var(--green)' },
          { label: '死亡', value: profile.statistics.deaths, color: 'var(--danger)' },
          { label: 'TK', value: profile.statistics.teamkills, color: 'var(--danger)' },
          { label: '场次', value: profile.total_sessions },
        ].map(({ label, value, color, sub }) => (
          <div key={label} className="card" style={{ padding: '10px 14px', textAlign: 'center' }}>
            <div style={{ fontSize: 11, color: 'var(--text3)' }}>{label}</div>
            <div style={{ fontSize: 24, fontWeight: 700, color: color || 'var(--text)' }}>{value}</div>
            {sub && <div style={{ fontSize: 11, color: 'var(--text3)' }}>{sub}</div>}
          </div>
        ))}
      </div>

      {/* ── Tab 导航 ── */}
      <div className="card">
        <div style={{ display: 'flex', borderBottom: '2px solid var(--border)', overflow: 'auto' }}>
          {tabs.map(t => (
            <button key={t.id}
              onClick={() => setActiveTab(t.id)}
              style={{
                padding: '10px 16px', fontSize: 13, fontWeight: activeTab === t.id ? 600 : 400,
                color: activeTab === t.id ? 'var(--primary)' : 'var(--text3)',
                borderBottom: activeTab === t.id ? '2px solid var(--primary)' : '2px solid transparent',
                marginBottom: -2, background: 'none', borderTop: 'none', borderLeft: 'none', borderRight: 'none',
                cursor: 'pointer', whiteSpace: 'nowrap',
              }}
            >{t.label}</button>
          ))}
        </div>

        <div className="card-body" style={{ maxHeight: 500, overflow: 'auto' }}>
          {/* 聊天记录 */}
          {activeTab === 'chat' && (
            <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
              <thead><tr style={{ borderBottom: '2px solid var(--border)', textAlign: 'left' }}>
                <th style={{ padding: '8px 12px', color: 'var(--text3)', fontWeight: 500, width: 140 }}>时间</th>
                <th style={{ padding: '8px 12px', color: 'var(--text3)', fontWeight: 500, width: 60 }}>频道</th>
                <th style={{ padding: '8px 12px', color: 'var(--text3)', fontWeight: 500 }}>消息</th>
              </tr></thead>
              <tbody>
                {profile.chat_history.length === 0 ? (
                  <tr><td colSpan={3} style={{ padding: 40, textAlign: 'center', color: 'var(--text3)' }}>暂无聊天记录</td></tr>
                ) : profile.chat_history.map((c, i) => (
                  <tr key={i} style={{ borderBottom: '1px solid var(--border)' }}>
                    <td style={{ padding: '6px 12px', fontSize: 11, color: 'var(--text2)', whiteSpace: 'nowrap' }}>{formatDate(c.logged_at)}</td>
                    <td style={{ padding: '6px 12px' }}>
                      <span style={{
                        fontSize: 11, padding: '1px 6px', borderRadius: 3,
                        background: c.channel === 'All' ? 'var(--bg3)' : c.channel === 'Team' ? 'rgba(59,130,246,0.15)' : 'rgba(34,197,94,0.15)',
                        color: c.channel === 'All' ? 'var(--text2)' : c.channel === 'Team' ? '#3b82f6' : '#22c55e',
                      }}>{c.channel}</span>
                    </td>
                    <td style={{ padding: '6px 12px', wordBreak: 'break-all' }}>{c.message}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}

          {/* 战斗记录 */}
          {activeTab === 'combat' && (
            <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
              <thead><tr style={{ borderBottom: '2px solid var(--border)', textAlign: 'left' }}>
                <th style={{ padding: '8px 12px', color: 'var(--text3)', fontWeight: 500, width: 140 }}>时间</th>
                <th style={{ padding: '8px 12px', color: 'var(--text3)', fontWeight: 500, width: 60 }}>类型</th>
                <th style={{ padding: '8px 12px', color: 'var(--text3)', fontWeight: 500 }}>目标/来源</th>
                <th style={{ padding: '8px 12px', color: 'var(--text3)', fontWeight: 500, width: 120 }}>武器</th>
                <th style={{ padding: '8px 12px', color: 'var(--text3)', fontWeight: 500, width: 50 }}>伤害</th>
              </tr></thead>
              <tbody>
                {profile.combat_history.length === 0 ? (
                  <tr><td colSpan={5} style={{ padding: 40, textAlign: 'center', color: 'var(--text3)' }}>暂无战斗记录</td></tr>
                ) : profile.combat_history.map((c, i) => (
                  <tr key={i} style={{ borderBottom: '1px solid var(--border)' }}>
                    <td style={{ padding: '6px 12px', fontSize: 11, color: 'var(--text2)', whiteSpace: 'nowrap' }}>{formatDate(c.event_time)}</td>
                    <td style={{ padding: '6px 12px' }}>
                      <span style={{
                        fontSize: 11, padding: '1px 6px', borderRadius: 3,
                        background: c.event_type === 'teamkill' ? 'rgba(239,68,68,0.15)' : c.is_attacker ? 'rgba(34,197,94,0.15)' : 'rgba(239,68,68,0.15)',
                        color: c.event_type === 'teamkill' ? 'var(--danger)' : c.is_attacker ? '#22c55e' : 'var(--danger)',
                      }}>{c.event_type === 'teamkill' ? 'TK' : c.is_attacker ? '击杀' : '死亡'}</span>
                    </td>
                    <td style={{ padding: '6px 12px' }}>{c.other_name || c.other_steam64}</td>
                    <td style={{ padding: '6px 12px', fontSize: 12, color: 'var(--text2)' }}>{c.weapon}</td>
                    <td style={{ padding: '6px 12px', fontFamily: 'monospace' }}>{c.damage.toFixed(0)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}

          {/* 违规记录 */}
          {activeTab === 'violations' && (
            <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
              <thead><tr style={{ borderBottom: '2px solid var(--border)', textAlign: 'left' }}>
                <th style={{ padding: '8px 12px', color: 'var(--text3)', fontWeight: 500, width: 140 }}>时间</th>
                <th style={{ padding: '8px 12px', color: 'var(--text3)', fontWeight: 500, width: 60 }}>处罚</th>
                <th style={{ padding: '8px 12px', color: 'var(--text3)', fontWeight: 500, width: 80 }}>类别</th>
                <th style={{ padding: '8px 12px', color: 'var(--text3)', fontWeight: 500 }}>消息</th>
              </tr></thead>
              <tbody>
                {profile.violations.length === 0 ? (
                  <tr><td colSpan={4} style={{ padding: 40, textAlign: 'center', color: 'var(--text3)' }}>暂无违规记录</td></tr>
                ) : profile.violations.map((v, i) => (
                  <tr key={i} style={{ borderBottom: '1px solid var(--border)' }}>
                    <td style={{ padding: '6px 12px', fontSize: 11, color: 'var(--text2)', whiteSpace: 'nowrap' }}>{formatDate(v.logged_at)}</td>
                    <td style={{ padding: '6px 12px' }}>
                      <span style={{
                        fontSize: 11, padding: '1px 6px', borderRadius: 3, fontWeight: 600,
                        background: v.action_taken === 'BAN' ? 'rgba(239,68,68,0.15)' : v.action_taken === 'KICK' ? 'rgba(249,115,22,0.15)' : 'rgba(234,179,8,0.15)',
                        color: v.action_taken === 'BAN' ? 'var(--danger)' : v.action_taken === 'KICK' ? '#f97316' : '#eab308',
                      }}>{v.action_taken}</span>
                    </td>
                    <td style={{ padding: '6px 12px', fontSize: 12 }}>{v.category}</td>
                    <td style={{ padding: '6px 12px', wordBreak: 'break-all' }}>{v.message}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}

          {/* 误伤分析 */}
          {activeTab === 'teamkills' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))', gap: 12 }}>
                {[
                  { label: '总 TK', value: profile.teamkill_metrics.total_teamkills },
                  { label: 'TK/场', value: profile.teamkill_metrics.teamkills_per_session.toFixed(2) },
                  { label: 'TK 比例', value: `${tkPct}%` },
                  { label: '近7天 TK', value: profile.teamkill_metrics.recent_teamkills },
                ].map(({ label, value }) => (
                  <div key={label} className="card" style={{ padding: '12px', textAlign: 'center' }}>
                    <div style={{ fontSize: 11, color: 'var(--text3)' }}>{label}</div>
                    <div style={{ fontSize: 24, fontWeight: 700 }}>{value}</div>
                  </div>
                ))}
              </div>
              {/* 武器 TK 分布 */}
              {profile.weapon_stats.filter(w => w.teamkills > 0).length > 0 && (
                <div>
                  <div style={{ fontSize: 13, fontWeight: 600, marginBottom: 8 }}>TK 武器分布</div>
                  <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
                    <thead><tr style={{ borderBottom: '2px solid var(--border)', textAlign: 'left' }}>
                      <th style={{ padding: '8px 12px', color: 'var(--text3)', fontWeight: 500 }}>武器</th>
                      <th style={{ padding: '8px 12px', color: 'var(--text3)', fontWeight: 500, width: 80 }}>击杀</th>
                      <th style={{ padding: '8px 12px', color: 'var(--text3)', fontWeight: 500, width: 80 }}>TK</th>
                    </tr></thead>
                    <tbody>
                      {profile.weapon_stats.filter(w => w.teamkills > 0).map((w, i) => (
                        <tr key={i} style={{ borderBottom: '1px solid var(--border)' }}>
                          <td style={{ padding: '6px 12px' }}>{w.weapon}</td>
                          <td style={{ padding: '6px 12px', fontFamily: 'monospace' }}>{w.kills}</td>
                          <td style={{ padding: '6px 12px', fontFamily: 'monospace', color: 'var(--danger)' }}>{w.teamkills}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              )}
            </div>
          )}

          {/* 武器统计 */}
          {activeTab === 'weapons' && (
            <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
              <thead><tr style={{ borderBottom: '2px solid var(--border)', textAlign: 'left' }}>
                <th style={{ padding: '8px 12px', color: 'var(--text3)', fontWeight: 500 }}>武器</th>
                <th style={{ padding: '8px 12px', color: 'var(--text3)', fontWeight: 500, width: 80 }}>击杀</th>
                <th style={{ padding: '8px 12px', color: 'var(--text3)', fontWeight: 500, width: 80 }}>TK</th>
              </tr></thead>
              <tbody>
                {profile.weapon_stats.length === 0 ? (
                  <tr><td colSpan={3} style={{ padding: 40, textAlign: 'center', color: 'var(--text3)' }}>暂无武器数据</td></tr>
                ) : profile.weapon_stats.map((w, i) => (
                  <tr key={i} style={{ borderBottom: '1px solid var(--border)' }}>
                    <td style={{ padding: '6px 12px' }}>{w.weapon}</td>
                    <td style={{ padding: '6px 12px', fontFamily: 'monospace', color: 'var(--green)' }}>{w.kills}</td>
                    <td style={{ padding: '6px 12px', fontFamily: 'monospace', color: w.teamkills > 0 ? 'var(--danger)' : 'var(--text2)' }}>{w.teamkills}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}

          {/* 名称历史 */}
          {activeTab === 'names' && (
            <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
              <thead><tr style={{ borderBottom: '2px solid var(--border)', textAlign: 'left' }}>
                <th style={{ padding: '8px 12px', color: 'var(--text3)', fontWeight: 500 }}>名称</th>
                <th style={{ padding: '8px 12px', color: 'var(--text3)', fontWeight: 500, width: 100 }}>出现次数</th>
              </tr></thead>
              <tbody>
                {profile.name_history.map((n, i) => (
                  <tr key={i} style={{ borderBottom: '1px solid var(--border)' }}>
                    <td style={{ padding: '6px 12px', fontWeight: n.name === profile.player_name ? 600 : 400 }}>
                      {n.name} {n.name === profile.player_name && <span style={{ fontSize: 10, color: 'var(--primary)', marginLeft: 6 }}>(当前)</span>}
                    </td>
                    <td style={{ padding: '6px 12px', fontFamily: 'monospace' }}>{n.session_count}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </div>
    </div>
  );
}
