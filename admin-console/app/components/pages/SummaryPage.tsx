'use client';

export default function SummaryPage() {
  return (
    <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      <div className="stats-grid">
        <div className="stat-card">
          <div className="stat-header"><span className="stat-title">当前在线玩家</span></div>
          <div className="stat-value">128 / 200</div>
          <div className="stat-desc" style={{ color: 'var(--green)' }}>+12 较上小时</div>
        </div>
        <div className="stat-card">
          <div className="stat-header"><span className="stat-title">今日总访问</span></div>
          <div className="stat-value">1,234</div>
          <div className="stat-desc">独立 IP 访问量</div>
        </div>
        <div className="stat-card">
          <div className="stat-header"><span className="stat-title">违规行为拦截</span></div>
          <div className="stat-value">45</div>
          <div className="stat-desc" style={{ color: 'var(--red)' }}>需要关注日志</div>
        </div>
        <div className="stat-card">
          <div className="stat-header"><span className="stat-title">服务器负载</span></div>
          <div className="stat-value">34%</div>
          <div className="stat-desc">CPU 及内存状态健康</div>
        </div>
      </div>
      <div className="card">
        <div className="card-header"><div className="card-title">欢迎使用控制面板</div></div>
        <div className="card-body">请点击左侧菜单管理服务器和查看各项日志信息。</div>
      </div>
    </div>
  );
}
