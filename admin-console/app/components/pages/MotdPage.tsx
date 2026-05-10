'use client';

import { useState, useEffect, useCallback } from 'react';
import { useServers } from '../../lib/useServers';
import { api } from '../../lib/api';

interface Rule {
  id: number;
  title: string;
  description: string;
}

export default function MotdPage() {
  const { servers } = useServers();
  const [serverId, setServerId] = useState<number | null>(null);
  const [prefixText, setPrefixText] = useState('');
  const [suffixText, setSuffixText] = useState('\n进入服务器即表示同意遵守以上规则。祝您游戏愉快！');
  const [rules, setRules] = useState<Rule[]>([
    { id: 1, title: '禁止恶意TK', description: '故意击杀队友将被警告、踢出或封禁' },
    { id: 2, title: '尊重其他玩家', description: '禁止辱骂、歧视性言论和人身攻击' },
    { id: 3, title: '听从小队指挥', description: '配合小队长指挥，保持团队协作' },
    { id: 4, title: '禁止消极游戏', description: '禁止挂机、故意浪费载具和资源' },
  ]);
  const [previewContent, setPreviewContent] = useState('');
  const [ruleCount, setRuleCount] = useState(0);
  const [loadingPreview, setLoadingPreview] = useState(false);
  const [copied, setCopied] = useState(false);
  const [nextRuleId, setNextRuleId] = useState(5);

  useEffect(() => {
    if (servers.length > 0 && !serverId) setServerId(servers[0].id);
  }, [servers, serverId]);

  const addRule = () => {
    setRules([...rules, { id: nextRuleId, title: '', description: '' }]);
    setNextRuleId(nextRuleId + 1);
  };

  const removeRule = (id: number) => {
    setRules(rules.filter(r => r.id !== id));
  };

  const updateRule = (id: number, field: 'title' | 'description', value: string) => {
    setRules(rules.map(r => r.id === id ? { ...r, [field]: value } : r));
  };

  const refreshPreview = useCallback(async () => {
    setLoadingPreview(true);
    try {
      const body = {
        rules: rules.filter(r => r.title.trim()).map(r => ({
          id: r.id,
          server_id: serverId || 0,
          display_order: r.id,
          title: r.title,
          description: r.description,
          actions: [],
          sub_rules: [],
        })),
        prefix_text: prefixText,
        suffix_text: suffixText,
        include_descriptions: true,
      };
      const res = await api('/motd/preview', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      });
      const data = await res.json();
      setPreviewContent(data.motd || '');
      setRuleCount(data.rule_count || 0);
    } catch (e) {
      setPreviewContent('生成预览失败');
    } finally {
      setLoadingPreview(false);
    }
  }, [rules, prefixText, suffixText, serverId]);

  const copyToClipboard = async () => {
    if (!previewContent) await refreshPreview();
    if (previewContent) {
      await navigator.clipboard.writeText(previewContent);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div className="page-view" style={{ gap: 20 }}>
      <h1 style={{ fontSize: 20, fontWeight: 700 }}>MOTD 配置</h1>

      {/* 服务器选择 */}
      <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
        <span style={{ fontSize: 12, color: 'var(--text3)' }}>服务器：</span>
        <select className="rcon-input" style={{ width: 'auto', padding: '6px 10px' }}
          value={serverId || ''} onChange={e => setServerId(parseInt(e.target.value))}>
          {servers.map(s => <option key={s.id} value={s.id}>{s.name}</option>)}
        </select>
      </div>

      {/* 前缀/后缀 */}
      <div className="card">
        <div className="card-header"><div className="card-title">内容配置</div></div>
        <div className="card-body" style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
          <div>
            <label style={{ fontSize: 13, fontWeight: 500, display: 'block', marginBottom: 4 }}>前缀文本</label>
            <textarea className="rcon-input" rows={3} style={{ width: '100%', resize: 'vertical' }}
              value={prefixText} onChange={e => setPrefixText(e.target.value)}
              placeholder="显示在规则前的内容（如服务器名称、欢迎语）" />
          </div>
          <div>
            <label style={{ fontSize: 13, fontWeight: 500, display: 'block', marginBottom: 4 }}>后缀文本</label>
            <textarea className="rcon-input" rows={3} style={{ width: '100%', resize: 'vertical' }}
              value={suffixText} onChange={e => setSuffixText(e.target.value)}
              placeholder="显示在规则后的内容" />
          </div>
        </div>
      </div>

      {/* 规则编辑 */}
      <div className="card">
        <div className="card-header">
          <div className="card-title">服务器规则</div>
          <button className="rcon-btn" style={{ width: 'auto', padding: '4px 12px', fontSize: 12 }}
            onClick={addRule}>+ 添加规则</button>
        </div>
        <div className="card-body" style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
          {rules.length === 0 && (
            <div style={{ textAlign: 'center', color: 'var(--text3)', padding: 20 }}>暂无规则，点击"添加规则"创建</div>
          )}
          {rules.map((rule, i) => (
            <div key={rule.id} style={{ display: 'flex', gap: 12, alignItems: 'flex-start', padding: '8px 0', borderBottom: '1px solid var(--border)' }}>
              <span style={{ fontSize: 12, color: 'var(--text3)', minWidth: 24, paddingTop: 6 }}>#{i + 1}</span>
              <div style={{ flex: 1, display: 'flex', flexDirection: 'column', gap: 6 }}>
                <input className="rcon-input" style={{ fontSize: 13 }}
                  value={rule.title} onChange={e => updateRule(rule.id, 'title', e.target.value)}
                  placeholder="规则标题" />
                <input className="rcon-input" style={{ fontSize: 12 }}
                  value={rule.description} onChange={e => updateRule(rule.id, 'description', e.target.value)}
                  placeholder="规则描述（可选）" />
              </div>
              <button className="rcon-btn" style={{ width: 'auto', padding: '4px 8px', fontSize: 11, color: 'var(--danger)', borderColor: 'var(--danger)' }}
                onClick={() => removeRule(rule.id)}>删除</button>
            </div>
          ))}
        </div>
      </div>

      {/* 预览 */}
      <div className="card">
        <div className="card-header">
          <div className="card-title">预览</div>
          <div style={{ display: 'flex', gap: 8 }}>
            <button className="rcon-btn" style={{ width: 'auto', padding: '6px 12px', fontSize: 12 }}
              onClick={refreshPreview} disabled={loadingPreview}>
              {loadingPreview ? '生成中...' : '刷新预览'}
            </button>
            <button className="rcon-btn" style={{ width: 'auto', padding: '6px 12px', fontSize: 12, background: 'var(--primary)', color: '#fff' }}
              onClick={copyToClipboard}>
              {copied ? '已复制 ✓' : '复制内容'}
            </button>
          </div>
        </div>
        <div className="card-body">
          {previewContent ? (
            <>
              <pre style={{ background: 'var(--bg3)', padding: 16, borderRadius: 6, fontSize: 13, whiteSpace: 'pre-wrap', maxHeight: 500, overflow: 'auto', fontFamily: 'monospace' }}>{previewContent}</pre>
              <div style={{ fontSize: 12, color: 'var(--text3)', marginTop: 8 }}>包含 {ruleCount} 条规则</div>
            </>
          ) : (
            <div style={{ textAlign: 'center', color: 'var(--text3)', padding: 20 }}>点击"刷新预览"生成 MOTD 内容</div>
          )}
        </div>
      </div>
    </div>
  );
}
