'use client';

export function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', borderBottom: '1px solid var(--border)', paddingBottom: 6 }}>
      <span style={{ color: 'var(--text3)', fontSize: 11 }}>{label}</span>
      <span style={{ fontWeight: 600, fontSize: 12 }}>{value}</span>
    </div>
  );
}
