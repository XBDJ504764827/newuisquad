'use client';

export default function FlyLogsPage() {
  return (
    <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      <div className="card">
        <div className="card-header">
          <div><div className="card-title">飞天 (Noclip) 使用记录</div><div className="card-sub">监控玩家及管理员的飞天指令使用情况。</div></div>
        </div>
        <div style={{ overflowX: 'auto' }}>
          <table>
            <thead>
              <tr><th>使用时间</th><th>玩家名称</th><th>SteamID64</th><th>操作类型</th></tr>
            </thead>
            <tbody>
              <tr><td>2026-05-01 09:12:33</td><td>Admin_Super</td><td>76561198123456789</td><td><span className="badge green">开启飞天 (noclip on)</span></td></tr>
              <tr><td>2026-05-01 09:15:01</td><td>Admin_Super</td><td>76561198123456789</td><td><span className="badge gray">关闭飞天 (noclip off)</span></td></tr>
              <tr><td>2026-05-01 10:50:11</td><td>王小虎</td><td>76561198000000001</td><td><span className="badge red">非法开启 (拦截)</span></td></tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
