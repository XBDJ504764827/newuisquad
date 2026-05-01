'use client';

export default function ConfigFilePage() {
  return (
    <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      <div className="card">
        <div className="card-header">
          <div><div className="card-title">配置文件编辑器</div><div className="card-sub">直接在网页上修改游戏服务器的底层配置文件。</div></div>
          <button className="rcon-btn" style={{ padding: '6px 12px', fontSize: 12, width: 'auto' }}>保存修改</button>
        </div>
        <div className="file-editor-layout" style={{ border: 'none', borderTop: '1px solid var(--border)', borderRadius: 0 }}>
          <div className="file-tree">
            <div className="file-item active">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/><polyline points="13 2 13 9 20 9"/></svg>
              {' '}server.cfg
            </div>
            <div className="file-item">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/><polyline points="13 2 13 9 20 9"/></svg>
              {' '}mapcycle.txt
            </div>
            <div className="file-item">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/><polyline points="13 2 13 9 20 9"/></svg>
              {' '}admins.json
            </div>
            <div className="file-item">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/><polyline points="13 2 13 9 20 9"/></svg>
              {' '}bans.cfg
            </div>
          </div>
          <div className="editor-area">
            <div className="editor-header">
              <span style={{ fontWeight: 500, fontSize: 12 }}>server.cfg</span>
              <span style={{ fontSize: 11, color: 'var(--text3)' }}>上次修改: 2小时前</span>
            </div>
            <textarea className="editor-textarea" defaultValue={`// 基础服务器配置
hostname "华东区-狂欢生存服"
rcon_password "******"
sv_password ""

// 游戏性调整
mp_friendlyfire 0
mp_maxrounds 30
mp_timelimit 40
mp_roundtime 3
sv_cheats 0

// 网络与速率
sv_minrate 20000
sv_maxrate 0
sv_minupdaterate 30
sv_maxupdaterate 64

log on`} />
          </div>
        </div>
      </div>
    </div>
  );
}
