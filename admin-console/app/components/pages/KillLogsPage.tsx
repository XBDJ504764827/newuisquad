'use client';

export default function KillLogsPage() {
  return (
    <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      <div className="card">
        <div className="card-header">
          <div><div className="card-title">击倒与击杀记录</div><div className="card-sub">详细追踪服务器内的所有战斗日志。</div></div>
        </div>
        <div style={{ overflowX: 'auto' }}>
          <table>
            <thead>
              <tr><th>时间</th><th>地图</th><th>攻击者 (角色)</th><th>攻击者 SteamID64</th><th>使用武器</th><th>受害者 (阵营)</th></tr>
            </thead>
            <tbody>
              <tr><td>10:15:22</td><td>de_dust2</td><td>张无忌 (突击手)</td><td>76561198111222333</td><td>AK-47</td><td>李大牛 <span className="badge red">反叛者</span></td></tr>
              <tr><td>10:18:05</td><td>de_dust2</td><td>李白 (狙击手)</td><td>76561198444555666</td><td>AWP</td><td>黄蓉 <span className="badge blue">特种部队</span></td></tr>
              <tr><td>10:22:11</td><td>de_dust2</td><td>陈奕 (支援)</td><td>76561198777888999</td><td>M4A1</td><td>张无忌 <span className="badge blue">特种部队</span></td></tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
