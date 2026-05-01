'use client';

import { useState } from 'react';

const configTabs = [
  { id: 'tab-1', label: '快捷设置' },
  { id: 'tab-2', label: '误杀设置' },
  { id: 'tab-3', label: '挂机设置' },
  { id: 'tab-4', label: '跳边设置' },
  { id: 'tab-5', label: '广播设置' },
  { id: 'tab-6', label: '队伍设置' },
  { id: 'tab-7', label: '悬赏设置' },
  { id: 'tab-8', label: '暖服设置' },
  { id: 'tab-9', label: '伤害通知' },
  { id: 'tab-10', label: '异常伤害' },
];

export default function ConfigPanelPage() {
  const [activeTab, setActiveTab] = useState('tab-1');

  return (
    <div className="page-view" style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      <div className="card">
        <div className="tabs-header">
          {configTabs.map((tab) => (
            <button
              key={tab.id}
              className={`tab-btn${activeTab === tab.id ? ' active' : ''}`}
              onClick={() => setActiveTab(tab.id)}
            >
              {tab.label}
            </button>
          ))}
        </div>

        {activeTab === 'tab-1' && (
          <div className="tab-content" style={{ display: 'block' }}>
            <h4 style={{ marginBottom: 16 }}>快捷设置</h4>
            <p style={{ color: 'var(--text3)', fontSize: 12 }}>此处可快速开关常用的插件和服务器基础功能。</p>
            <div style={{ marginTop: 20, display: 'flex', flexDirection: 'column', gap: 16 }}>
              <label style={{ display: 'flex', alignItems: 'center', gap: 10 }}><input type="checkbox" defaultChecked /> 启用服务器密码保护</label>
              <label style={{ display: 'flex', alignItems: 'center', gap: 10 }}><input type="checkbox" defaultChecked /> 开启反作弊模块</label>
              <label style={{ display: 'flex', alignItems: 'center', gap: 10 }}><input type="checkbox" /> 允许全员语音跨队</label>
            </div>
          </div>
        )}

        {activeTab === 'tab-2' && <div className="tab-content" style={{ display: 'block' }}><h4>误杀设置 (TK)</h4><p style={{ color: 'var(--text3)', fontSize: 12, marginTop: 8 }}>设置队友误伤惩罚和自动踢出机制。（功能UI待实现）</p></div>}
        {activeTab === 'tab-3' && <div className="tab-content" style={{ display: 'block' }}><h4>挂机设置 (AFK)</h4><p style={{ color: 'var(--text3)', fontSize: 12, marginTop: 8 }}>设置挂机检测时间及处理方式。（功能UI待实现）</p></div>}
        {activeTab === 'tab-4' && <div className="tab-content" style={{ display: 'block' }}><h4>跳边设置</h4><p style={{ color: 'var(--text3)', fontSize: 12, marginTop: 8 }}>限制玩家频繁更换阵营。（功能UI待实现）</p></div>}
        {activeTab === 'tab-5' && <div className="tab-content" style={{ display: 'block' }}><h4>广播设置</h4><p style={{ color: 'var(--text3)', fontSize: 12, marginTop: 8 }}>配置定时系统广播消息。（功能UI待实现）</p></div>}
        {activeTab === 'tab-6' && <div className="tab-content" style={{ display: 'block' }}><h4>队伍设置</h4><p style={{ color: 'var(--text3)', fontSize: 12, marginTop: 8 }}>配置队伍人数上限、自动平衡规则等。（功能UI待实现）</p></div>}
        {activeTab === 'tab-7' && <div className="tab-content" style={{ display: 'block' }}><h4>悬赏设置</h4><p style={{ color: 'var(--text3)', fontSize: 12, marginTop: 8 }}>击杀连杀玩家的赏金系统设置。（功能UI待实现）</p></div>}
        {activeTab === 'tab-8' && <div className="tab-content" style={{ display: 'block' }}><h4>暖服设置</h4><p style={{ color: 'var(--text3)', fontSize: 12, marginTop: 8 }}>设置人少时的 BOT 添加或特定暖服地图。（功能UI待实现）</p></div>}
        {activeTab === 'tab-9' && <div className="tab-content" style={{ display: 'block' }}><h4>伤害通知</h4><p style={{ color: 'var(--text3)', fontSize: 12, marginTop: 8 }}>HUD 伤害显示开关及样式。（功能UI待实现）</p></div>}
        {activeTab === 'tab-10' && <div className="tab-content" style={{ display: 'block' }}><h4>异常伤害</h4><p style={{ color: 'var(--text3)', fontSize: 12, marginTop: 8 }}>定义过高伤害的检测阈值及反制措施。（功能UI待实现）</p></div>}
      </div>
    </div>
  );
}
