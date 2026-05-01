'use client';

export default function ActionLogsPage() {
  return (
    <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      <div className="card">
        <div className="card-header">
          <div><div className="card-title">网页端操作审计记录</div><div className="card-sub">记录所有网站后台管理员的操作日志。</div></div>
        </div>
        <div style={{ overflowX: 'auto' }}>
          <table>
            <thead>
              <tr><th>操作时间</th><th>管理员账户</th><th>操作模块</th><th>操作详情</th><th>IP 地址</th></tr>
            </thead>
            <tbody>
              <tr><td>2026-05-01 10:45:00</td><td>Admin</td><td>登录系统</td><td>管理员成功登录面板</td><td>192.168.1.100</td></tr>
              <tr><td>2026-05-01 10:48:12</td><td>Admin</td><td>配置文件</td><td>修改并保存了 server.cfg</td><td>192.168.1.100</td></tr>
              <tr><td>2026-05-01 10:52:05</td><td>Moderator_01</td><td>控制面板</td><td>执行 RCON: kick 王小虎</td><td>10.0.0.55</td></tr>
              <tr><td>2026-05-01 10:58:30</td><td>Admin</td><td>配置面板</td><td>关闭了 快捷设置-&gt;允许跨队语音</td><td>192.168.1.100</td></tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
