'use client';

export default function PermissionsPage() {
  return (
    <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      <div className="card">
        <div className="card-header">
          <div><div className="card-title">后台权限矩阵</div><div className="card-sub">在此分配和管理不同网站用户的模块访问权限。</div></div>
          <button className="rcon-btn" style={{ padding: '6px 12px', fontSize: 12, width: 'auto' }}>保存权限分配</button>
        </div>
        <div style={{ overflowX: 'auto' }}>
          <table>
            <thead>
              <tr>
                <th>用户角色</th>
                <th style={{ textAlign: 'center' }}>控制面板</th>
                <th style={{ textAlign: 'center' }}>日志查询</th>
                <th style={{ textAlign: 'center' }}>修改配置</th>
                <th style={{ textAlign: 'center' }}>玩家管理</th>
                <th style={{ textAlign: 'center' }}>权限分配</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td><strong>超级管理员 (Super Admin)</strong></td>
                <td style={{ textAlign: 'center' }}><input type="checkbox" defaultChecked disabled /></td>
                <td style={{ textAlign: 'center' }}><input type="checkbox" defaultChecked disabled /></td>
                <td style={{ textAlign: 'center' }}><input type="checkbox" defaultChecked disabled /></td>
                <td style={{ textAlign: 'center' }}><input type="checkbox" defaultChecked disabled /></td>
                <td style={{ textAlign: 'center' }}><input type="checkbox" defaultChecked disabled /></td>
              </tr>
              <tr>
                <td><strong>服主协管 (Moderator)</strong></td>
                <td style={{ textAlign: 'center' }}><input type="checkbox" defaultChecked /></td>
                <td style={{ textAlign: 'center' }}><input type="checkbox" defaultChecked /></td>
                <td style={{ textAlign: 'center' }}><input type="checkbox" /></td>
                <td style={{ textAlign: 'center' }}><input type="checkbox" defaultChecked /></td>
                <td style={{ textAlign: 'center' }}><input type="checkbox" /></td>
              </tr>
              <tr>
                <td><strong>巡查员 (Inspector)</strong></td>
                <td style={{ textAlign: 'center' }}><input type="checkbox" /></td>
                <td style={{ textAlign: 'center' }}><input type="checkbox" defaultChecked /></td>
                <td style={{ textAlign: 'center' }}><input type="checkbox" /></td>
                <td style={{ textAlign: 'center' }}><input type="checkbox" /></td>
                <td style={{ textAlign: 'center' }}><input type="checkbox" /></td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
