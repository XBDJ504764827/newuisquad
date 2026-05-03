'use client';

import { useState, useEffect } from 'react';
import { useServers } from '../../lib/useServers';
import { api } from '../../lib/api';
import Pagination from '../Pagination';

interface MatchEvent {
    id: number;
    server_id: number;
    map_name: string;
    layer_name: string;
    team1_faction: string;
    team2_faction: string;
    winner_team: number | null;
    event_type: string;
    logged_at: string;
}

const MAP_NAMES: Record<string, string> = {
    'Anvil': '铁砧',
    'AlBasrah': '巴士拉',
    'BlackCoast': '黑海岸',
    'Chora': '乔拉',
    'Fallujah': '费卢杰',
    'FoolsRoad': '愚者之路',
    'GooseBay': '鹅湾',
    'Gorodok': '格罗多克',
    'Harju': '哈尔尤',
    'Kamdesh': '卡姆德什',
    'Kohat': '科哈特',
    'Kokan': '科坎',
    'Lashkar': '拉什卡尔',
    'Logar': '洛加尔',
    'Manicouagan': '马尼夸根',
    'Mestia': '梅斯蒂亚',
    'Mutaha': '穆塔哈',
    'Narva': '纳尔瓦',
    'Sanxian_Islands': '三仙岛',
    'Skorpo': '斯科尔波',
    'Sumari': '苏马里',
    'Tallil': '塔利尔',
    'Yehorivka': '耶霍里夫卡',
};

const EVENT_LABELS: Record<string, string> = {
    'layer_change': '切换地图',
    'match_start': '比赛开始',
    'match_end': '比赛结束',
};

function translateMap(name: string): string {
    for (const [key, val] of Object.entries(MAP_NAMES)) {
        if (name.includes(key)) return val;
    }
    return name;
}

function formatLayer(layer: string): string {
    return layer.replace(/_/g, ' ').replace(/ v\d+$/, '');
}

function factionFlag(faction: string): string {
    if (faction.includes('PLA') || faction.includes("People's Liberation Army")) return '🇨🇳';
    if (faction.includes('US Army') || faction.includes('United States Army')) return '🇺🇸';
    if (faction.includes('British') || faction.includes('British Army')) return '🇬🇧';
    if (faction.includes('Canadian')) return '🇨🇦';
    if (faction.includes('Australian')) return '🇦🇺';
    if (faction.includes('Russian') || faction.includes('Russian Ground Forces')) return '🇷🇺';
    if (faction.includes('Insurgent') || faction.includes('Irregular')) return '🏴';
    if (faction.includes('Turkish')) return '🇹🇷';
    return '🎖️';
}

export default function MatchLogsPage() {
    const { servers } = useServers();
    const [serverId, setServerId] = useState<number | null>(null);
    const [events, setEvents] = useState<MatchEvent[]>([]);
    const [page, setPage] = useState(1);
    const [total, setTotal] = useState(0);
    const [loading, setLoading] = useState(false);
    const [expandedMatch, setExpandedMatch] = useState<number | null>(null);

    useEffect(() => {
        if (servers.length > 0 && !serverId) setServerId(servers[0].id);
    }, [servers, serverId]);

    useEffect(() => {
        if (!serverId) return;
        setLoading(true);
        api(`/servers/${serverId}/match-events?page=${page}&per_page=20`)
            .then(r => r.json())
            .then(d => { setEvents(d.data || []); setTotal(d.total || 0); setLoading(false); })
            .catch(e => { console.error(e); setLoading(false); });
    }, [serverId, page]);

    // 将事件按比赛分组（相近时间的事件归为同一场比赛）
    const matches = groupMatches(events);

    return (
        <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
            <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
                <span style={{ fontSize: 12, color: 'var(--text3)' }}>服务器：</span>
                <select className="rcon-input" style={{ width: 'auto', padding: '6px 10px' }}
                    value={serverId || ''}
                    onChange={e => { setServerId(parseInt(e.target.value)); setPage(1); }}>
                    {servers.map(s => <option key={s.id} value={s.id}>{s.name}</option>)}
                </select>
            </div>

            <div className="card">
                <div className="card-header">
                    <div>
                        <div className="card-title">比赛记录</div>
                        <div className="card-sub">地图切换与比赛结果时间线（共 {total} 条）</div>
                    </div>
                </div>
                <div className="card-body" style={{ padding: 0 }}>
                    {loading ? (
                        <div style={{ padding: 40, textAlign: 'center', color: 'var(--text3)' }}>加载中...</div>
                    ) : events.length === 0 ? (
                        <div className="empty-state">
                            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
                                <circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/>
                            </svg>
                            <h3>暂无比赛记录</h3>
                            <p style={{ marginTop: 8 }}>游戏服务器运行后将自动记录比赛数据。</p>
                        </div>
                    ) : (
                        <div style={{ padding: '16px 20px' }}>
                            {/* 时间线 */}
                            <div style={{ position: 'relative', paddingLeft: 32 }}>
                                {/* 时间线竖线 */}
                                <div style={{
                                    position: 'absolute', left: 11, top: 8, bottom: 8,
                                    width: 2, background: 'var(--border)',
                                }} />

                                {matches.map((matchGroup, mi) => {
                                    const mainEvent = matchGroup[0];
                                    const isExpanded = expandedMatch === mi;
                                    const mapName = translateMap(mainEvent.map_name);
                                    const layer = formatLayer(mainEvent.layer_name);
                                    const winnerTeam = mainEvent.winner_team;

                                    return (
                                        <div key={mi} style={{ position: 'relative', marginBottom: mi < matches.length - 1 ? 24 : 0 }}>
                                            {/* 时间线圆点 */}
                                            <div style={{
                                                position: 'absolute', left: -25, top: 6,
                                                width: 10, height: 10, borderRadius: '50%',
                                                background: winnerTeam ? 'var(--green, #22c55e)' : '#f59e0b',
                                                border: '2px solid var(--bg2)',
                                                zIndex: 1,
                                            }} />

                                            {/* 时间戳 */}
                                            <div style={{ fontSize: 11, color: 'var(--text3)', marginBottom: 6 }}>
                                                {new Date(mainEvent.logged_at).toLocaleString()}
                                            </div>

                                            {/* 比赛卡片 */}
                                            <div
                                                onClick={() => setExpandedMatch(isExpanded ? null : mi)}
                                                style={{
                                                    background: 'var(--bg3)',
                                                    border: `1px solid ${winnerTeam ? 'rgba(34,197,94,0.2)' : 'var(--border)'}`,
                                                    borderRadius: 'var(--radius)',
                                                    padding: '14px 16px',
                                                    cursor: 'pointer',
                                                    transition: 'border-color 0.2s',
                                                }}
                                            >
                                                {/* 头部：地图 + 比分 */}
                                                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                                                    <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                                                        <span style={{ fontSize: 16, fontWeight: 700 }}>{mapName}</span>
                                                        <span style={{ fontSize: 12, color: 'var(--text2)', background: 'var(--bg2)', padding: '2px 8px', borderRadius: 4 }}>{layer}</span>
                                                        {winnerTeam && (
                                                            <span style={{ fontSize: 11, background: 'rgba(34,197,94,0.15)', color: '#22c55e', padding: '2px 8px', borderRadius: 4 }}>
                                                                已结束
                                                            </span>
                                                        )}
                                                    </div>
                                                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"
                                                        style={{ transform: isExpanded ? 'rotate(180deg)' : 'none', transition: 'transform 0.2s', flexShrink: 0 }}>
                                                        <polyline points="6 9 12 15 18 9"/>
                                                    </svg>
                                                </div>

                                                {/* 对阵双方 */}
                                                <div style={{
                                                    display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 12,
                                                    marginTop: 10, padding: '10px 0',
                                                    borderTop: '1px solid var(--border)',
                                                }}>
                                                    <div style={{ textAlign: 'center', flex: 1 }}>
                                                        <div style={{ fontSize: 20 }}>{factionFlag(mainEvent.team1_faction)}</div>
                                                        <div style={{ fontSize: 11, fontWeight: 600, marginTop: 2, color: winnerTeam === 1 ? '#22c55e' : 'var(--text)' }}>
                                                            {winnerTeam === 1 ? '👑 ' : ''}{mainEvent.team1_faction}
                                                        </div>
                                                    </div>
                                                    <div style={{ fontSize: 18, fontWeight: 800, color: 'var(--text3)' }}>VS</div>
                                                    <div style={{ textAlign: 'center', flex: 1 }}>
                                                        <div style={{ fontSize: 20 }}>{factionFlag(mainEvent.team2_faction)}</div>
                                                        <div style={{ fontSize: 11, fontWeight: 600, marginTop: 2, color: winnerTeam === 2 ? '#22c55e' : 'var(--text)' }}>
                                                            {winnerTeam === 2 ? '👑 ' : ''}{mainEvent.team2_faction}
                                                        </div>
                                                    </div>
                                                </div>

                                                {/* 展开的事件详情 */}
                                                {isExpanded && matchGroup.length > 1 && (
                                                    <div style={{ borderTop: '1px solid var(--border)', marginTop: 8, paddingTop: 8 }}>
                                                        {matchGroup.slice(1).map((evt, ei) => (
                                                            <div key={ei} style={{
                                                                display: 'flex', alignItems: 'center', gap: 8,
                                                                padding: '4px 0', fontSize: 12,
                                                            }}>
                                                                <span style={{
                                                                    width: 6, height: 6, borderRadius: '50%',
                                                                    background: evt.event_type === 'match_end' ? '#22c55e' : '#f59e0b',
                                                                    flexShrink: 0,
                                                                }} />
                                                                <span style={{ color: 'var(--text2)', whiteSpace: 'nowrap' }}>
                                                                    {new Date(evt.logged_at).toLocaleTimeString()}
                                                                </span>
                                                                <span>{EVENT_LABELS[evt.event_type] || evt.event_type}</span>
                                                            </div>
                                                        ))}
                                                    </div>
                                                )}
                                            </div>
                                        </div>
                                    );
                                })}
                            </div>

                            <Pagination page={page} total={total} perPage={20} onPageChange={setPage} />
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}

// 将连续事件按时间窗口（15分钟内）分组为单场比赛
function groupMatches(events: MatchEvent[]): MatchEvent[][] {
    if (events.length === 0) return [];
    const groups: MatchEvent[][] = [];
    let currentGroup: MatchEvent[] = [];
    const threshold = 15 * 60 * 1000; // 15分钟

    // events 按时间倒序排列，反转后处理
    const sorted = [...events].reverse();

    for (const evt of sorted) {
        if (currentGroup.length === 0) {
            currentGroup.push(evt);
        } else {
            const lastTime = new Date(currentGroup[currentGroup.length - 1].logged_at).getTime();
            const thisTime = new Date(evt.logged_at).getTime();
            if (thisTime - lastTime < threshold) {
                currentGroup.push(evt);
            } else {
                groups.push(currentGroup);
                currentGroup = [evt];
            }
        }
    }
    if (currentGroup.length > 0) groups.push(currentGroup);

    // 每组反转回倒序（最新的在前），且整个 groups 也反转
    return groups.map(g => g.reverse()).reverse();
}
