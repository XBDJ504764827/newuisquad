'use client';

import { useState, useEffect } from 'react';
import { useServers } from '../../lib/useServers';
import { api } from '../../lib/api';

interface Summary {
    player_count_24h: number;
    error_count_24h: number;
    warn_count_24h: number;
    match_count_24h: number;
    kill_count_24h: number;
    tk_count_24h: number;
    latest_match: {
        map_name: string;
        layer_name: string;
        team1_faction: string;
        team2_faction: string;
        winner_team: number | null;
        logged_at: string;
    } | null;
    recent_errors: Array<{ level: string; message: string; logged_at: string }>;
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

export default function SummaryPage() {
    const { servers } = useServers();
    const [serverId, setServerId] = useState<number | null>(null);
    const [summary, setSummary] = useState<Summary | null>(null);
    const [loading, setLoading] = useState(false);
    const [heatmapData, setHeatmapData] = useState<Array<{x: number; y: number; count: number}>>([]);
    const [heatmapLoading, setHeatmapLoading] = useState(false);

    useEffect(() => {
        if (servers.length > 0 && !serverId) setServerId(servers[0].id);
    }, [servers, serverId]);

    useEffect(() => {
        if (!serverId) return;
        setLoading(true);
        api(`/servers/${serverId}/summary`)
            .then(r => r.json())
            .then(d => { setSummary(d); setLoading(false); })
            .catch(e => { console.error(e); setLoading(false); });

        // 加载爆炸热力图数据
        setHeatmapLoading(true);
        api(`/servers/${serverId}/explosion-events?per_page=500`)
            .then(r => r.json())
            .then(d => {
                const points = (d.data || []).map((e: any) => ({
                    x: e.pos_x, y: e.pos_y, count: 1,
                }));
                // 合并相近坐标点
                const merged: Record<string, {x: number; y: number; count: number}> = {};
                for (const p of points) {
                    if (p.x === 0 && p.y === 0) continue;
                    const key = `${Math.round(p.x / 500) * 500},${Math.round(p.y / 500) * 500}`;
                    if (!merged[key]) merged[key] = { x: Math.round(p.x / 500) * 500, y: Math.round(p.y / 500) * 500, count: 0 };
                    merged[key].count++;
                }
                setHeatmapData(Object.values(merged));
                setHeatmapLoading(false);
            })
            .catch(e => { console.error(e); setHeatmapLoading(false); });
    }, [serverId]);

    const errorRate = summary ? Math.round(summary.error_count_24h / Math.max(1, summary.match_count_24h) * 100) / 100 : 0;
    const tkRate = summary ? Math.round(summary.tk_count_24h / Math.max(1, summary.kill_count_24h) * 1000) / 10 : 0;

    if (loading && !summary) {
        return (
            <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
                <div className="card"><div className="card-body" style={{ textAlign: 'center', color: 'var(--text3)', padding: 40 }}>加载中...</div></div>
            </div>
        );
    }

    return (
        <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
            {/* 服务器选择 */}
            <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
                <span style={{ fontSize: 12, color: 'var(--text3)' }}>服务器：</span>
                <select className="rcon-input" style={{ width: 'auto', padding: '6px 10px' }}
                    value={serverId || ''}
                    onChange={e => setServerId(parseInt(e.target.value))}>
                    {servers.map(s => <option key={s.id} value={s.id}>{s.name}</option>)}
                </select>
            </div>

            {/* 统计卡片 */}
            <div className="stats-grid">
                <div className="stat-card">
                    <div className="stat-header"><span className="stat-title">24h 独立玩家</span></div>
                    <div className="stat-value">{summary?.player_count_24h || 0}</div>
                    <div className="stat-desc" style={{ color: 'var(--green)' }}>去过重 Steam64</div>
                </div>
                <div className="stat-card">
                    <div className="stat-header"><span className="stat-title">24h 比赛场次</span></div>
                    <div className="stat-value">{summary?.match_count_24h || 0}</div>
                    <div className="stat-desc">地图切换次数</div>
                </div>
                <div className="stat-card">
                    <div className="stat-header"><span className="stat-title">24h 击杀 / TK</span></div>
                    <div className="stat-value">{summary?.kill_count_24h || 0}</div>
                    <div className="stat-desc" style={{ color: tkRate > 5 ? 'var(--red)' : 'var(--text2)' }}>
                        TK率 {tkRate}% {tkRate > 5 ? '⚠ 偏高' : ''}
                    </div>
                </div>
                <div className="stat-card">
                    <div className="stat-header"><span className="stat-title">24h 错误率</span></div>
                    <div className="stat-value" style={{ color: errorRate > 10 ? 'var(--red)' : 'inherit' }}>{summary?.error_count_24h || 0}</div>
                    <div className="stat-desc" style={{ color: summary && summary.error_count_24h > 50 ? 'var(--red)' : 'var(--text2)' }}>
                        警告 {summary?.warn_count_24h || 0} 条
                    </div>
                </div>
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 20 }}>
                {/* 最近比赛 */}
                <div className="card">
                    <div className="card-header">
                        <div><div className="card-title">最近比赛</div></div>
                    </div>
                    <div className="card-body">
                        {summary?.latest_match ? (
                            <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
                                <div style={{ fontSize: 14, fontWeight: 600 }}>
                                    {summary.latest_match.map_name} - {summary.latest_match.layer_name.replace(/_/g, ' ')}
                                </div>
                                <div style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 13 }}>
                                    <span>{factionFlag(summary.latest_match.team1_faction)} {summary.latest_match.team1_faction}</span>
                                    <span style={{ color: 'var(--text3)' }}>VS</span>
                                    <span>{factionFlag(summary.latest_match.team2_faction)} {summary.latest_match.team2_faction}</span>
                                </div>
                                <div style={{ fontSize: 11, color: 'var(--text3)' }}>
                                    {new Date(summary.latest_match.logged_at).toLocaleString()}
                                    {summary.latest_match.winner_team && (
                                        <span style={{ marginLeft: 8, color: '#22c55e' }}>
                                            👑 队伍 {summary.latest_match.winner_team} 获胜
                                        </span>
                                    )}
                                </div>
                            </div>
                        ) : (
                            <div style={{ color: 'var(--text3)', textAlign: 'center', padding: 20 }}>暂无比赛数据</div>
                        )}
                    </div>
                </div>

                {/* 最近错误/告警 */}
                <div className="card">
                    <div className="card-header">
                        <div><div className="card-title">最近错误 / 警告</div></div>
                    </div>
                    <div className="card-body" style={{ padding: 0 }}>
                        {summary?.recent_errors?.length ? (
                            <div style={{ maxHeight: 250, overflowY: 'auto' }}>
                                {summary.recent_errors.map((e, i) => (
                                    <div key={i} style={{
                                        display: 'flex', gap: 8, padding: '8px 14px',
                                        borderBottom: '1px solid var(--border)',
                                        fontSize: 12, alignItems: 'flex-start',
                                    }}>
                                        <span style={{
                                            flexShrink: 0, padding: '1px 6px', borderRadius: 4, fontSize: 10,
                                            background: e.level === 'ERROR' ? 'rgba(239,68,68,0.15)' : 'rgba(245,158,11,0.15)',
                                            color: e.level === 'ERROR' ? 'var(--red)' : '#f59e0b',
                                        }}>
                                            {e.level}
                                        </span>
                                        <span style={{ flex: 1, wordBreak: 'break-word' }}>{e.message.length > 120 ? e.message.slice(0, 120) + '...' : e.message}</span>
                                        <span style={{ flexShrink: 0, color: 'var(--text3)', fontSize: 11, whiteSpace: 'nowrap' }}>
                                            {new Date(e.logged_at).toLocaleTimeString()}
                                        </span>
                                    </div>
                                ))}
                            </div>
                        ) : (
                            <div style={{ color: 'var(--green)', textAlign: 'center', padding: 24, fontSize: 13 }}>✅ 无错误或告警，服务器运行正常</div>
                        )}
                    </div>
                </div>
            </div>

            {/* 爆炸热力图 */}
            <div className="card">
                <div className="card-header">
                    <div>
                        <div className="card-title">爆炸热力图</div>
                        <div className="card-sub">最近爆炸事件分布（{heatmapData.length} 个热点区域）</div>
                    </div>
                    {heatmapLoading && <span style={{ fontSize: 11, color: 'var(--text3)' }}>加载中...</span>}
                </div>
                <div className="card-body" style={{ padding: 12 }}>
                    {heatmapData.length > 0 ? (
                        <ExplosionHeatmap points={heatmapData} />
                    ) : (
                        <div style={{ textAlign: 'center', color: 'var(--text3)', padding: 40 }}>
                            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" style={{ marginBottom: 12 }}>
                                <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="16"/><line x1="8" y1="12" x2="16" y2="12"/>
                            </svg>
                            <p>暂无爆炸数据</p>
                            <p style={{ fontSize: 11, marginTop: 4 }}>Agent 将在解析游戏日志后自动填充爆炸事件坐标</p>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}

// Canvas 热力图组件
function ExplosionHeatmap({ points }: { points: Array<{x: number; y: number; count: number}> }) {
    useEffect(() => {
        const canvas = document.getElementById('heatmap-canvas') as HTMLCanvasElement;
        if (!canvas) return;
        const ctx = canvas.getContext('2d');
        if (!ctx) return;

        const dpr = window.devicePixelRatio || 1;
        const rect = canvas.getBoundingClientRect();
        canvas.width = rect.width * dpr;
        canvas.height = rect.height * dpr;
        ctx.scale(dpr, dpr);

        // 清除
        ctx.fillStyle = '#0a0a0f';
        ctx.fillRect(0, 0, rect.width, rect.height);

        if (points.length === 0) return;

        // 计算坐标范围
        const xs = points.map(p => p.x);
        const ys = points.map(p => p.y);
        const xMin = Math.min(...xs), xMax = Math.max(...xs);
        const yMin = Math.min(...ys), yMax = Math.max(...ys);
        const xRange = (xMax - xMin) || 1, yRange = (yMax - yMin) || 1;
        const maxCount = Math.max(...points.map(p => p.count));

        const pad = 30;
        const w = rect.width - pad * 2, h = rect.height - pad * 2;

        // 绘制热点
        for (const p of points) {
            const sx = pad + ((p.x - xMin) / xRange) * w;
            const sy = pad + ((p.y - yMin) / yRange) * h;
            const intensity = p.count / maxCount;
            const radius = 3 + intensity * 18;

            const gradient = ctx.createRadialGradient(sx, sy, 0, sx, sy, radius);
            if (intensity > 0.7) {
                gradient.addColorStop(0, `rgba(255, 50, 50, ${0.8 * intensity})`);
                gradient.addColorStop(0.5, `rgba(255, 150, 30, ${0.5 * intensity})`);
            } else if (intensity > 0.3) {
                gradient.addColorStop(0, `rgba(255, 150, 30, ${0.6 * intensity})`);
                gradient.addColorStop(0.5, `rgba(255, 200, 50, ${0.3 * intensity})`);
            } else {
                gradient.addColorStop(0, `rgba(255, 200, 50, ${0.4 * intensity})`);
                gradient.addColorStop(0.5, `rgba(255, 255, 100, ${0.15 * intensity})`);
            }
            gradient.addColorStop(1, 'rgba(255, 255, 100, 0)');

            ctx.beginPath();
            ctx.arc(sx, sy, radius, 0, Math.PI * 2);
            ctx.fillStyle = gradient;
            ctx.fill();
        }

        // 坐标轴标注
        ctx.fillStyle = '#666';
        ctx.font = '9px monospace';
        ctx.textAlign = 'center';
        ctx.fillText(`${(xMin / 100).toFixed(0)}k`, pad, rect.height - 8);
        ctx.fillText(`${(xMax / 100).toFixed(0)}k`, pad + w, rect.height - 8);
        ctx.textAlign = 'left';
        ctx.fillText(`${(yMin / 100).toFixed(0)}k`, 2, pad + 4);
        ctx.fillText(`${(yMax / 100).toFixed(0)}k`, 2, pad + h);

        // 热力等级图例
        const lx = rect.width - 90, ly = 8;
        const stops = [
            { label: '高', color: '#ff3232' },
            { label: '中', color: '#ff9620' },
            { label: '低', color: '#ffc832' },
        ];
        stops.forEach((s, i) => {
            const sy = ly + i * 14;
            ctx.fillStyle = s.color;
            ctx.fillRect(lx, sy, 12, 10);
            ctx.fillStyle = '#888';
            ctx.font = '9px sans-serif';
            ctx.textAlign = 'left';
            ctx.fillText(s.label, lx + 16, sy + 9);
        });
    }, [points]);

    return (
        <canvas id="heatmap-canvas" style={{
            width: '100%', height: 280, borderRadius: 8,
            background: '#0a0a0f',
        }} />
    );
}
