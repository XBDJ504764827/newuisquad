'use client';

export default function ChatLogsPage() {
  return (
    <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      <div className="card">
        <div className="card-header">
          <div><div className="card-title">全局聊天记录</div><div className="card-sub">实时拦截和记录服务器内的玩家通讯内容。</div></div>
        </div>
        <div style={{ overflowX: 'auto' }}>
          <table>
            <thead>
              <tr><th>发送时间</th><th>发送者</th><th>接收目标</th><th>聊天内容</th></tr>
            </thead>
            <tbody>
              <tr><td>2026-05-01 10:48:05</td><td>李大牛</td><td><span className="badge gray">全局</span></td><td>大家好啊！</td></tr>
              <tr><td>2026-05-01 10:49:12</td><td>张无忌</td><td><span className="badge blue">队伍 (A队)</span></td><td>快过来帮我架枪，我被架住了！</td></tr>
              <tr><td>2026-05-01 10:52:30</td><td>陈奕</td><td><span className="badge gray">全局</span></td><td>有人换装备吗？</td></tr>
              <tr><td>2026-05-01 10:55:01</td><td>黄蓉</td><td>私聊 -&gt; 张无忌</td><td>我马上到你那里。</td></tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
