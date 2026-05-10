'use client';

import { useState, useEffect, useCallback, useRef } from 'react';
import { api } from '../../lib/api';

interface Server { id: number; server_id: string; name: string; ip: string; rcon_port: number; }
interface RuleAction {
  id: string; violation_count: number; action_type: 'WARN' | 'KICK' | 'BAN';
  duration_days: number; message: string;
}
interface RuleItem {
  id: string; parent_id: string | null; display_order: number;
  title: string; description: string;
  actions: RuleAction[]; sub_rules: RuleItem[];
  _displayId?: string;
}

let _uuidCounter = 0;
function genId(): string { _uuidCounter++; return `rule_${Date.now()}_${_uuidCounter}`; }

function annotateNumbers(rules: RuleItem[]) {
  rules.forEach((r, i) => {
    r._displayId = String(i + 1);
    (r.sub_rules || []).forEach((sr, si) => { sr._displayId = `${i + 1}.${si + 1}`; });
  });
}

function flattenRules(rules: RuleItem[]): RuleItem[] {
  const result: RuleItem[] = [];
  for (const r of rules) {
    const { sub_rules, ...rest } = r;
    result.push(rest as RuleItem);
    if (sub_rules?.length) result.push(...flattenRules(sub_rules));
  }
  return result;
}

function generateText(rules: RuleItem[]): string {
  let text = '';
  rules.forEach((r, i) => {
    text += `${i + 1}. ${r.title}\n`;
    if (r.description?.trim()) text += `    * ${r.description}\n`;
    (r.sub_rules || []).forEach((sr, si) => {
      text += `    ${i + 1}.${si + 1}. ${sr.title}\n`;
      if (sr.description?.trim()) text += `        * ${sr.description}\n`;
    });
    text += '\n';
  });
  return text.trim();
}

export default function ServerRulesPage() {
  const [servers, setServers] = useState<Server[]>([]);
  const [serverId, setServerId] = useState<number | null>(null);
  const [rules, setRules] = useState<RuleItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [hasChanges, setHasChanges] = useState(false);
  const [saving, setSaving] = useState(false);
  const [previewOpen, setPreviewOpen] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    api('/servers').then(r => r.json()).then(d => {
      const list = d.data || [];
      setServers(list);
      if (list.length > 0) setServerId(list[0].id);
    }).catch(() => {});
  }, []);

  const loadRules = useCallback(async (sid: number) => {
    setLoading(true);
    setError(null);
    try {
      const res = await api(`/servers/${sid}/files/list`);
      const data = await res.json();
      const files = data.data || data.files || [];
      const rulesFile = files.find((f: any) => f.name?.toLowerCase().includes('rule') || f.path?.toLowerCase().includes('rule'));
      if (rulesFile) {
        const fileRes = await api(`/servers/${sid}/files?path=${encodeURIComponent(rulesFile.path || rulesFile.name)}`);
        const fileData = await fileRes.json();
        const content = fileData.data || fileData.content || '';
        try {
          const parsed = JSON.parse(content);
          if (Array.isArray(parsed)) { setRules(parsed); annotateNumbers(parsed); }
        } catch { setRules([]); }
      } else {
        setRules([]);
      }
    } catch (e: any) { setError(e.message); }
    setLoading(false);
  }, []);

  useEffect(() => { if (serverId) loadRules(serverId); }, [serverId, loadRules]);

  const saveRules = async () => {
    if (!serverId) return;
    setSaving(true);
    setError(null);
    try {
      const flat = flattenRules(rules);
      // Save as JSON file via files API
      await api(`/servers/${serverId}/files`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path: 'Rules.json', content: JSON.stringify(flat, null, 2) }),
      });
      // Also generate text version
      const textContent = generateText(rules);
      await api(`/servers/${serverId}/files`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path: 'Rules.txt', content: textContent }),
      });
      setHasChanges(false);
    } catch (e: any) { setError(e.message); }
    setSaving(false);
  };

  const addRule = () => {
    const newRule: RuleItem = { id: genId(), parent_id: null, display_order: rules.length, title: `规则 ${rules.length + 1}`, description: '', actions: [], sub_rules: [] };
    const updated = [...rules, newRule];
    setRules(updated);
    annotateNumbers(updated);
    setHasChanges(true);
  };

  const addSubRule = (parentId: string) => {
    const fn = (list: RuleItem[]): RuleItem[] => list.map(r => {
      if (r.id === parentId) {
        const subs = r.sub_rules || [];
        const sr: RuleItem = { id: genId(), parent_id: r.id, display_order: subs.length, title: `子规则 ${subs.length + 1}`, description: '', actions: [], sub_rules: [] };
        return { ...r, sub_rules: [...subs, sr] };
      }
      if (r.sub_rules?.length) return { ...r, sub_rules: fn(r.sub_rules) };
      return r;
    });
    const updated = fn(rules);
    setRules(updated);
    annotateNumbers(updated);
    setHasChanges(true);
  };

  const updateRule = (ruleId: string, field: string, value: any) => {
    const fn = (list: RuleItem[]): RuleItem[] => list.map(r => {
      if (r.id === ruleId) return { ...r, [field]: value };
      if (r.sub_rules?.length) return { ...r, sub_rules: fn(r.sub_rules) };
      return r;
    });
    const updated = fn(rules);
    setRules(updated);
    setHasChanges(true);
  };

  const deleteRule = (ruleId: string) => {
    const fn = (list: RuleItem[]): RuleItem[] => list.filter(r => {
      if (r.id === ruleId) return false;
      if (r.sub_rules?.length) r.sub_rules = fn(r.sub_rules);
      return true;
    });
    const updated = fn(rules);
    setRules(updated);
    annotateNumbers(updated);
    setHasChanges(true);
  };

  const addAction = (ruleId: string) => {
    const action: RuleAction = { id: genId(), violation_count: 1, action_type: 'WARN', duration_days: 0, message: '' };
    const fn = (list: RuleItem[]): RuleItem[] => list.map(r => {
      if (r.id === ruleId) return { ...r, actions: [...(r.actions || []), action] };
      if (r.sub_rules?.length) return { ...r, sub_rules: fn(r.sub_rules) };
      return r;
    });
    const updated = fn(rules);
    setRules(updated);
    setHasChanges(true);
  };

  const updateAction = (ruleId: string, actionId: string, field: string, value: any) => {
    const fn = (list: RuleItem[]): RuleItem[] => list.map(r => {
      if (r.id === ruleId) return { ...r, actions: (r.actions || []).map(a => a.id === actionId ? { ...a, [field]: value } : a) };
      if (r.sub_rules?.length) return { ...r, sub_rules: fn(r.sub_rules) };
      return r;
    });
    const updated = fn(rules);
    setRules(updated);
    setHasChanges(true);
  };

  const deleteAction = (ruleId: string, actionId: string) => {
    const fn = (list: RuleItem[]): RuleItem[] => list.map(r => {
      if (r.id === ruleId) return { ...r, actions: (r.actions || []).filter(a => a.id !== actionId) };
      if (r.sub_rules?.length) return { ...r, sub_rules: fn(r.sub_rules) };
      return r;
    });
    const updated = fn(rules);
    setRules(updated);
    setHasChanges(true);
  };

  const handleImport = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (ev) => {
      const content = ev.target?.result as string;
      try {
        if (file.name.endsWith('.json')) {
          const imported = JSON.parse(content);
          if (Array.isArray(imported)) { setRules(imported); annotateNumbers(imported); setHasChanges(true); }
        } else {
          // Parse text format
          const lines = content.split('\n').filter(l => l.trim());
          const result: RuleItem[] = [];
          let current: RuleItem | null = null;
          lines.forEach(line => {
            const trimmed = line.trim();
            const main = trimmed.match(/^(\d+)\.\s*(.+)$/);
            if (main) {
              if (current) result.push(current);
              current = { id: genId(), parent_id: null, display_order: result.length, title: main[2], description: '', actions: [], sub_rules: [] };
              return;
            }
            const sub = trimmed.match(/^(\d+\.\d+)\.\s*(.+)$/);
            if (sub && current) {
              current.sub_rules.push({ id: genId(), parent_id: current.id, display_order: current.sub_rules.length, title: sub[2], description: '', actions: [], sub_rules: [] });
              return;
            }
            const desc = trimmed.match(/^\*\s*(.+)$/);
            if (desc && current?.sub_rules?.length) {
              const last = current.sub_rules[current.sub_rules.length - 1];
              last.description = desc[1];
            }
          });
          if (current) result.push(current);
          setRules(result);
          annotateNumbers(result);
          setHasChanges(true);
        }
      } catch {}
    };
    reader.readAsText(file);
    if (fileInputRef.current) fileInputRef.current.value = '';
  };

  const exportJson = () => {
    const flat = flattenRules(rules);
    const blob = new Blob([JSON.stringify(flat, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a'); a.href = url; a.download = 'server-rules.json'; a.click();
    URL.revokeObjectURL(url);
  };

  const exportText = () => {
    const text = generateText(rules);
    const blob = new Blob([text], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a'); a.href = url; a.download = 'server-rules.txt'; a.click();
    URL.revokeObjectURL(url);
  };

  const styles = {
    container: { padding: 20 },
    header: { display: 'flex', alignItems: 'center', gap: 12, marginBottom: 20, flexWrap: 'wrap' as const },
    select: { padding: '8px 12px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--bg2)', color: 'var(--text)', fontSize: 14, minWidth: 200 },
    btn: { padding: '8px 16px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--text)', color: 'var(--bg)', cursor: 'pointer', fontWeight: 500, fontSize: 13 },
    btnSm: { padding: '4px 10px', borderRadius: 4, border: '1px solid var(--border)', background: 'var(--bg2)', color: 'var(--text)', cursor: 'pointer', fontSize: 11 },
    btnDanger: { padding: '4px 10px', borderRadius: 4, border: '1px solid rgba(239,68,68,0.3)', background: 'rgba(239,68,68,0.1)', color: '#ef4444', cursor: 'pointer', fontSize: 11 },
    ruleCard: { background: 'var(--bg2)', borderRadius: 10, border: '1px solid var(--border)', padding: 16, marginBottom: 12 },
    ruleInput: { padding: '8px 12px', borderRadius: 6, border: '1px solid var(--border)', background: '#1e1e1e', color: 'var(--text)', fontSize: 13, width: '100%', boxSizing: 'border-box' as const },
    actionCard: { background: 'rgba(0,0,0,0.15)', borderRadius: 6, padding: '10px 14px', marginTop: 8 },
    actionSelect: { padding: '4px 8px', borderRadius: 4, border: '1px solid var(--border)', background: '#1e1e1e', color: 'var(--text)', fontSize: 12 },
  };

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h2 style={{ margin: 0, fontSize: 20, fontWeight: 600 }}>服务器规则管理</h2>
        <select value={serverId || ''} onChange={e => setServerId(Number(e.target.value))} style={styles.select}>
          {servers.map(s => <option key={s.id} value={s.id}>{s.name} ({s.ip}:{s.rcon_port})</option>)}
        </select>
        <button onClick={addRule} style={styles.btn}>新增规则</button>
        <input type="file" ref={fileInputRef} onChange={handleImport} accept=".json,.txt" style={{ display: 'none' }} />
        <button onClick={() => fileInputRef.current?.click()} style={{ ...styles.btn, background: 'var(--bg2)', color: 'var(--text)' }}>导入</button>
        <button onClick={exportText} style={{ ...styles.btn, background: 'var(--bg2)', color: 'var(--text)' }}>导出TXT</button>
        <button onClick={exportJson} style={{ ...styles.btn, background: 'var(--bg2)', color: 'var(--text)' }}>导出JSON</button>
        <button onClick={() => setPreviewOpen(!previewOpen)} style={{ ...styles.btn, background: 'var(--bg2)', color: 'var(--text)' }}>{previewOpen ? '隐藏预览' : '预览'}</button>
        <button onClick={saveRules} disabled={!hasChanges || saving} style={{ ...styles.btn, opacity: (!hasChanges || saving) ? 0.5 : 1 }}>{saving ? '保存中...' : '保存规则'}</button>
      </div>

      {error && <div style={{ padding: '8px 12px', background: 'rgba(239,68,68,0.1)', color: '#ef4444', borderRadius: 6, marginBottom: 12, fontSize: 13 }}>{error}</div>}
      {hasChanges && <div style={{ padding: '8px 12px', background: 'rgba(234,179,8,0.1)', color: '#eab308', borderRadius: 6, marginBottom: 12, fontSize: 12 }}>有未保存的更改</div>}

      {loading && <div style={{ textAlign: 'center', padding: 30, color: 'var(--text3)' }}>加载中...</div>}

      {previewOpen && (
        <div style={{ background: 'var(--bg2)', borderRadius: 10, border: '1px solid var(--border)', padding: 16, marginBottom: 20 }}>
          <h4 style={{ margin: '0 0 8px 0', fontSize: 14, color: 'var(--text)' }}>规则预览</h4>
          <pre style={{ background: '#1e1e1e', color: '#d4d4d4', padding: 14, borderRadius: 6, margin: 0, fontSize: 12, whiteSpace: 'pre-wrap', maxHeight: 300, overflow: 'auto', fontFamily: 'monospace' }}>{generateText(rules) || '暂无规则'}</pre>
        </div>
      )}

      {!loading && rules.length === 0 && (
        <div style={{ textAlign: 'center', padding: 40, color: 'var(--text3)' }}>
          <p style={{ fontSize: 16, marginBottom: 8 }}>暂无规则</p>
          <p style={{ fontSize: 13 }}>点击"新增规则"或"导入"来添加服务器规则</p>
        </div>
      )}

      {rules.map((rule, _idx) => (
        <RuleCard
          key={rule.id}
          rule={rule}
          depth={0}
          onUpdate={updateRule}
          onDelete={deleteRule}
          onAddSubRule={addSubRule}
          onAddAction={addAction}
          onUpdateAction={updateAction}
          onDeleteAction={deleteAction}
          styles={styles}
        />
      ))}
    </div>
  );
}

function RuleCard({ rule, depth, onUpdate, onDelete, onAddSubRule, onAddAction, onUpdateAction, onDeleteAction, styles }: {
  rule: RuleItem; depth: number;
  onUpdate: (id: string, f: string, v: any) => void;
  onDelete: (id: string) => void;
  onAddSubRule: (id: string) => void;
  onAddAction: (id: string) => void;
  onUpdateAction: (ruleId: string, actionId: string, f: string, v: any) => void;
  onDeleteAction: (ruleId: string, actionId: string) => void;
  styles: any;
}) {
  const prefix = rule._displayId || '';
  const ml = depth * 24;

  return (
    <div style={{ marginLeft: ml }}>
      <div style={styles.ruleCard}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8, flexWrap: 'wrap' }}>
          <strong style={{ fontSize: 14, color: 'var(--text)', minWidth: 30 }}>{prefix}</strong>
          <input
            value={rule.title}
            onChange={e => onUpdate(rule.id, 'title', e.target.value)}
            style={{ ...styles.ruleInput, flex: 1, minWidth: 200 }}
            placeholder="规则标题"
          />
          <button onClick={() => onAddSubRule(rule.id)} style={styles.btnSm}>+子规则</button>
          <button onClick={() => onAddAction(rule.id)} style={styles.btnSm}>+动作</button>
          <button onClick={() => onDelete(rule.id)} style={styles.btnDanger}>删除</button>
        </div>
        <input
          value={rule.description || ''}
          onChange={e => onUpdate(rule.id, 'description', e.target.value)}
          style={{ ...styles.ruleInput, marginBottom: 8 }}
          placeholder="规则描述（可选）"
        />

        {/* Actions */}
        {(rule.actions || []).map(action => (
          <div key={action.id} style={styles.actionCard}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap' }}>
              <span style={{ fontSize: 11, color: 'var(--text3)' }}>第</span>
              <input
                type="number"
                value={action.violation_count}
                onChange={e => onUpdateAction(rule.id, action.id, 'violation_count', parseInt(e.target.value) || 1)}
                style={{ width: 50, padding: '4px 6px', borderRadius: 4, border: '1px solid var(--border)', background: '#1e1e1e', color: 'var(--text)', fontSize: 12, textAlign: 'center' }}
                min={1}
              />
              <span style={{ fontSize: 11, color: 'var(--text3)' }}>次违规 →</span>
              <select value={action.action_type} onChange={e => onUpdateAction(rule.id, action.id, 'action_type', e.target.value)} style={styles.actionSelect}>
                <option value="WARN">警告</option>
                <option value="KICK">踢出</option>
                <option value="BAN">封禁</option>
              </select>
              {action.action_type === 'BAN' && (
                <>
                  <span style={{ fontSize: 11, color: 'var(--text3)' }}>封禁</span>
                  <input
                    type="number"
                    value={action.duration_days || 0}
                    onChange={e => onUpdateAction(rule.id, action.id, 'duration_days', parseInt(e.target.value) || 0)}
                    style={{ width: 50, padding: '4px 6px', borderRadius: 4, border: '1px solid var(--border)', background: '#1e1e1e', color: 'var(--text)', fontSize: 12, textAlign: 'center' }}
                    min={0}
                  />
                  <span style={{ fontSize: 11, color: 'var(--text3)' }}>天</span>
                </>
              )}
              <input
                value={action.message || ''}
                onChange={e => onUpdateAction(rule.id, action.id, 'message', e.target.value)}
                style={{ flex: 1, minWidth: 100, padding: '4px 8px', borderRadius: 4, border: '1px solid var(--border)', background: '#1e1e1e', color: 'var(--text)', fontSize: 12 }}
                placeholder="动作消息（可选）"
              />
              <button onClick={() => onDeleteAction(rule.id, action.id)} style={{ ...styles.btnDanger, padding: '2px 6px', fontSize: 10 }}>×</button>
            </div>
          </div>
        ))}
      </div>

      {/* Sub-rules */}
      {(rule.sub_rules || []).map(sr => (
        <RuleCard
          key={sr.id}
          rule={sr}
          depth={depth + 1}
          onUpdate={onUpdate}
          onDelete={onDelete}
          onAddSubRule={onAddSubRule}
          onAddAction={onAddAction}
          onUpdateAction={onUpdateAction}
          onDeleteAction={onDeleteAction}
          styles={styles}
        />
      ))}
    </div>
  );
}
