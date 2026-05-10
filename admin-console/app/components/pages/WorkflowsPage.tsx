'use client';

import { useState, useEffect, useCallback } from 'react';
import { useServers } from '../../lib/useServers';
import { api } from '../../lib/api';

interface WorkflowDef { version: string; triggers: any[]; steps: any[]; }
interface Workflow { id: number; server_id: number; name: string; description: string; enabled: boolean; definition: WorkflowDef; created_by: string; created_at: string; updated_at: string; }
interface Execution { id: number; workflow_id: number; status: string; trigger_event_type: string; trigger_data: any; started_at: string; completed_at: string | null; error_message: string | null; }

export default function WorkflowsPage() {
  const { servers } = useServers();
  const [serverId, setServerId] = useState<number | null>(null);
  const [workflows, setWorkflows] = useState<Workflow[]>([]);
  const [loading, setLoading] = useState(false);

  // 列表创建/编辑
  const [showModal, setShowModal] = useState(false);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [form, setForm] = useState({ name: '', description: '', enabled: true, definition: '{"version":"1.0","triggers":[],"steps":[]}' });
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');

  // 定义编辑器（选中行展开）
  const [editingWf, setEditingWf] = useState<Workflow | null>(null);
  const [defText, setDefText] = useState('');
  const [defSaving, setDefSaving] = useState(false);
  const [defError, setDefError] = useState('');
  const [defSuccess, setDefSuccess] = useState('');

  // 执行记录
  const [showExecs, setShowExecs] = useState<number | null>(null);
  const [executions, setExecutions] = useState<Execution[]>([]);
  const [execLoading, setExecLoading] = useState(false);

  useEffect(() => { if (servers.length > 0 && !serverId) setServerId(servers[0].id); }, [servers, serverId]);

  const fetchWorkflows = useCallback(async () => {
    if (!serverId) return;
    setLoading(true);
    try { const res = await api(`/servers/${serverId}/workflows`); const d = await res.json(); setWorkflows(d.data || []); } catch {}
    setLoading(false);
  }, [serverId]);

  useEffect(() => { fetchWorkflows(); }, [fetchWorkflows]);

  const openCreate = () => {
    setEditingId(null); setForm({ name: '', description: '', enabled: true, definition: '{"version":"1.0","triggers":[],"steps":[]}' }); setError(''); setShowModal(true);
  };
  const openEdit = (w: Workflow) => {
    setEditingId(w.id); setForm({ name: w.name, description: w.description, enabled: w.enabled, definition: JSON.stringify(w.definition, null, 2) }); setError(''); setShowModal(true);
  };
  const save = async () => {
    if (!form.name.trim()) { setError('名称不能为空'); return; }
    let defJson: any; try { defJson = JSON.parse(form.definition); } catch { setError('定义 JSON 格式无效'); return; }
    setSaving(true);
    try {
      const body = { name: form.name.trim(), description: form.description, enabled: form.enabled, definition: defJson };
      const url = editingId ? `/servers/${serverId}/workflows/${editingId}` : `/servers/${serverId}/workflows`;
      const res = await api(url, { method: editingId ? 'PUT' : 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
      const d = await res.json();
      if (d.error) { setError(d.error); return; }
      setShowModal(false); fetchWorkflows();
    } catch (e: any) { setError(e.message); }
    setSaving(false);
  };
  const toggleEnabled = async (w: Workflow) => {
    try {
      const res = await api(`/servers/${serverId}/workflows/${w.id}/toggle`, { method: 'POST' });
      const d = await res.json();
      if (d.success) setWorkflows(prev => prev.map(wf => wf.id === w.id ? { ...wf, enabled: d.enabled } : wf));
    } catch {}
  };
  const deleteWorkflow = async (id: number, name: string) => {
    if (!confirm(`确定要删除工作流 "${name}" 吗？`)) return;
    try { await api(`/servers/${serverId}/workflows/${id}`, { method: 'DELETE' }); fetchWorkflows(); } catch {}
  };

  // 定义编辑器
  const openDefEditor = (w: Workflow) => {
    setEditingWf(w); setDefText(JSON.stringify(w.definition, null, 2)); setDefError(''); setDefSuccess(''); setShowExecs(null);
  };
  const saveDef = async () => {
    if (!editingWf) return;
    let defJson: any; try { defJson = JSON.parse(defText); } catch { setDefError('JSON 格式无效'); return; }
    setDefSaving(true); setDefError(''); setDefSuccess('');
    try {
      const res = await api(`/servers/${serverId}/workflows/${editingWf.id}`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ definition: defJson }) });
      const d = await res.json();
      if (d.error) { setDefError(d.error); return; }
      setEditingWf(d.data); setDefText(JSON.stringify(d.data.definition, null, 2)); setDefSuccess('保存成功');
      fetchWorkflows();
      setTimeout(() => setDefSuccess(''), 2000);
    } catch (e: any) { setDefError(e.message); }
    setDefSaving(false);
  };
  const fetchExecs = async (wfId: number) => {
    setShowExecs(wfId === showExecs ? null : wfId);
    if (wfId !== showExecs) {
      setExecLoading(true);
      try { const res = await api(`/servers/${serverId}/workflows/${wfId}/executions`); const d = await res.json(); setExecutions(d.data || []); } catch {}
      setExecLoading(false);
    }
  };

  const styles = {
    btn: { padding: '6px 14px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--bg2)', color: 'var(--text)', cursor: 'pointer', fontSize: 12 },
    btnPrimary: { padding: '8px 16px', borderRadius: 6, border: '1px solid var(--accent)', background: 'var(--text)', color: 'var(--bg)', cursor: 'pointer', fontWeight: 500, fontSize: 13 },
    card: { background: 'var(--bg2)', borderRadius: 10, border: '1px solid var(--border)', padding: 16, marginBottom: 12 },
    th: { padding: '8px 12px', textAlign: 'left' as const, fontWeight: 500, color: 'var(--text3)', fontSize: 12, borderBottom: '2px solid var(--border)' },
    td: { padding: '8px 12px', color: 'var(--text2)', fontSize: 12, borderBottom: '1px solid var(--border)' },
    badge: (bg: string, c: string) => ({ display: 'inline-block', padding: '2px 10px', borderRadius: 10, fontSize: 11, fontWeight: 600, background: bg, color: c }),
    input: { padding: '8px 12px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--bg)', color: 'var(--text)', fontSize: 13, width: '100%', boxSizing: 'border-box' as const },
    select: { padding: '6px 12px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--bg2)', color: 'var(--text)', fontSize: 13 },
  };

  return (
    <div style={{ padding: 20 }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16, flexWrap: 'wrap', gap: 12 }}>
        <h2 style={{ margin: 0, fontSize: 20, fontWeight: 600 }}>工作流管理</h2>
        <button onClick={openCreate} style={styles.btnPrimary}>+ 新建工作流</button>
      </div>

      <div style={{ display: 'flex', gap: 8, alignItems: 'center', marginBottom: 16 }}>
        <select value={serverId || ''} onChange={e => setServerId(parseInt(e.target.value))} style={styles.select}>
          {servers.map(s => <option key={s.id} value={s.id}>{s.name}</option>)}
        </select>
        <button onClick={fetchWorkflows} disabled={loading} style={styles.btn}>{loading ? '加载中...' : '刷新'}</button>
      </div>

      {/* 工作流列表 */}
      <div style={{ ...styles.card, padding: 0, overflow: 'hidden' }}>
        {workflows.length === 0 ? (
          <div style={{ textAlign: 'center', padding: 40, color: 'var(--text3)' }}>{loading ? '加载中...' : '暂无工作流'}</div>
        ) : (
          <div style={{ overflow: 'auto' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
              <thead><tr>
                <th style={styles.th}>名称</th><th style={styles.th}>描述</th><th style={styles.th}>步骤</th><th style={styles.th}>状态</th><th style={styles.th}>更新时间</th><th style={{ ...styles.th, textAlign: 'right' }}>操作</th>
              </tr></thead>
              <tbody>
                {workflows.map(w => (
                  <tr key={w.id} style={{ borderBottom: '1px solid var(--border)', background: editingWf?.id === w.id ? 'rgba(59,130,246,0.05)' : 'transparent' }}>
                    <td style={{ ...styles.td, fontWeight: 600 }}>{w.name}</td>
                    <td style={{ ...styles.td, maxWidth: 200, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{w.description || '-'}</td>
                    <td style={{ ...styles.td, textAlign: 'center', fontFamily: 'monospace' }}>{w.definition?.steps?.length || 0}</td>
                    <td style={styles.td}>
                      <span style={styles.badge(w.enabled ? 'rgba(34,197,94,0.15)' : 'rgba(156,163,175,0.15)', w.enabled ? '#22c55e' : 'var(--text3)')}>{w.enabled ? '启用' : '禁用'}</span>
                    </td>
                    <td style={{ ...styles.td, whiteSpace: 'nowrap' }}>{new Date(w.updated_at).toLocaleString('zh-CN')}</td>
                    <td style={{ ...styles.td, textAlign: 'right' }}>
                      <div style={{ display: 'flex', gap: 4, justifyContent: 'flex-end' }}>
                        <button onClick={() => toggleEnabled(w)} style={styles.btn}>{w.enabled ? '禁用' : '启用'}</button>
                        <button onClick={() => openEdit(w)} style={styles.btn}>编辑</button>
                        <button onClick={() => openDefEditor(w)} style={{ ...styles.btn, color: editingWf?.id === w.id ? 'var(--accent)' : 'var(--text)' }}>定义</button>
                        <button onClick={() => fetchExecs(w.id)} style={styles.btn}>记录</button>
                        <button onClick={() => deleteWorkflow(w.id, w.name)} style={{ ...styles.btn, color: '#ef4444', borderColor: 'rgba(239,68,68,0.3)' }}>删除</button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* 内联定义编辑器 */}
      {editingWf && (
        <div style={styles.card}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 12, flexWrap: 'wrap', gap: 8 }}>
            <h3 style={{ margin: 0, fontSize: 15 }}>编辑定义：{editingWf.name}</h3>
            <div style={{ display: 'flex', gap: 8 }}>
              <button onClick={saveDef} disabled={defSaving} style={styles.btnPrimary}>{defSaving ? '保存中...' : '保存定义'}</button>
              <button onClick={() => setEditingWf(null)} style={styles.btn}>关闭</button>
            </div>
          </div>
          {defError && <div style={{ padding: '8px 12px', borderRadius: 6, background: 'rgba(239,68,68,0.1)', color: '#ef4444', fontSize: 13, marginBottom: 8 }}>{defError}</div>}
          {defSuccess && <div style={{ padding: '8px 12px', borderRadius: 6, background: 'rgba(34,197,94,0.1)', color: '#22c55e', fontSize: 13, marginBottom: 8 }}>{defSuccess}</div>}
          <textarea style={{ ...styles.input, minHeight: 300, fontFamily: 'monospace', fontSize: 12, resize: 'vertical' }}
            value={defText} onChange={e => setDefText(e.target.value)} />
          <div style={{ marginTop: 12, fontSize: 12, color: 'var(--text3)' }}>
            触发器: {editingWf.definition?.triggers?.length || 0} · 步骤: {editingWf.definition?.steps?.length || 0}
          </div>
        </div>
      )}

      {/* 执行记录 */}
      {showExecs && (
        <div style={styles.card}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
            <h3 style={{ margin: 0, fontSize: 15 }}>执行记录</h3>
            <button onClick={() => fetchExecs(showExecs!)} style={styles.btn}>刷新</button>
          </div>
          {execLoading ? <div style={{ textAlign: 'center', color: 'var(--text3)', padding: 20 }}>加载中...</div> :
            executions.length === 0 ? <div style={{ textAlign: 'center', color: 'var(--text3)', padding: 20 }}>暂无执行记录</div> :
            <div style={{ overflow: 'auto' }}>
              <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
                <thead><tr>
                  <th style={styles.th}>ID</th><th style={styles.th}>状态</th><th style={styles.th}>触发事件</th><th style={styles.th}>开始时间</th><th style={styles.th}>完成时间</th><th style={styles.th}>错误</th>
                </tr></thead>
                <tbody>
                  {executions.map(e => (
                    <tr key={e.id} style={{ borderBottom: '1px solid var(--border)' }}>
                      <td style={{ ...styles.td, fontFamily: 'monospace', fontSize: 11 }}>{e.id}</td>
                      <td style={styles.td}>
                        <span style={styles.badge(
                          e.status === 'COMPLETED' ? 'rgba(34,197,94,0.15)' : e.status === 'FAILED' ? 'rgba(239,68,68,0.15)' : 'rgba(59,130,246,0.15)',
                          e.status === 'COMPLETED' ? '#22c55e' : e.status === 'FAILED' ? '#ef4444' : '#3b82f6'
                        )}>{e.status}</span>
                      </td>
                      <td style={styles.td}>{e.trigger_event_type || '-'}</td>
                      <td style={{ ...styles.td, whiteSpace: 'nowrap' }}>{new Date(e.started_at).toLocaleString('zh-CN')}</td>
                      <td style={{ ...styles.td, whiteSpace: 'nowrap' }}>{e.completed_at ? new Date(e.completed_at).toLocaleString('zh-CN') : '-'}</td>
                      <td style={{ ...styles.td, color: '#ef4444', maxWidth: 200, overflow: 'hidden', textOverflow: 'ellipsis' }}>{e.error_message || '-'}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          }
        </div>
      )}

      {/* 基础信息弹窗 */}
      {showModal && (
        <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.5)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 1000 }}>
          <div style={{ background: 'var(--bg)', borderRadius: 12, border: '1px solid var(--border)', width: '90vw', maxWidth: 560, maxHeight: '90vh', overflow: 'auto', padding: 20 }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 16 }}>
              <h3 style={{ margin: 0, fontSize: 16 }}>{editingId ? '编辑工作流' : '新建工作流'}</h3>
              <button onClick={() => setShowModal(false)} style={styles.btn}>✕</button>
            </div>
            {error && <div style={{ padding: '8px 12px', borderRadius: 6, background: 'rgba(239,68,68,0.1)', color: '#ef4444', fontSize: 13, marginBottom: 12 }}>{error}</div>}
            <div style={{ marginBottom: 12 }}>
              <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 4 }}>名称 *</label>
              <input value={form.name} onChange={e => setForm({ ...form, name: e.target.value })} style={styles.input} placeholder="工作流名称" />
            </div>
            <div style={{ marginBottom: 12 }}>
              <label style={{ fontSize: 12, color: 'var(--text2)', display: 'block', marginBottom: 4 }}>描述</label>
              <input value={form.description} onChange={e => setForm({ ...form, description: e.target.value })} style={styles.input} placeholder="可选描述" />
            </div>
            <label style={{ display: 'flex', alignItems: 'center', gap: 8, cursor: 'pointer', fontSize: 13, marginBottom: 16 }}>
              <input type="checkbox" checked={form.enabled} onChange={e => setForm({ ...form, enabled: e.target.checked })} /> 启用
            </label>
            <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
              <button onClick={() => setShowModal(false)} style={styles.btn}>取消</button>
              <button onClick={save} disabled={saving} style={{ ...styles.btnPrimary, opacity: saving ? 0.6 : 1 }}>{saving ? '保存中...' : '保存'}</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
