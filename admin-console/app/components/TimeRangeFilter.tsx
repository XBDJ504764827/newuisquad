'use client';

import { useState } from 'react';

interface TimeRangeFilterProps {
  onApply: (start: string, end: string) => void;
  onClear: () => void;
  hasFilter: boolean;
}

export default function TimeRangeFilter({ onApply, onClear, hasFilter }: TimeRangeFilterProps) {
  const [startTime, setStartTime] = useState('');
  const [endTime, setEndTime] = useState('');

  const handleApply = () => {
    onApply(startTime, endTime);
  };

  const handleClear = () => {
    setStartTime('');
    setEndTime('');
    onClear();
  };

  return (
    <div style={{ display: 'flex', gap: 8, alignItems: 'center', flexWrap: 'wrap' }}>
      <span style={{ fontSize: 12, color: 'var(--text3)' }}>时间范围:</span>
      <input
        type="datetime-local"
        value={startTime}
        onChange={e => setStartTime(e.target.value)}
        style={{
          padding: '5px 8px', borderRadius: 6, border: '1px solid var(--border)',
          background: 'var(--bg2)', color: 'var(--text1)', fontSize: 12,
        }}
      />
      <span style={{ fontSize: 12, color: 'var(--text3)' }}>至</span>
      <input
        type="datetime-local"
        value={endTime}
        onChange={e => setEndTime(e.target.value)}
        style={{
          padding: '5px 8px', borderRadius: 6, border: '1px solid var(--border)',
          background: 'var(--bg2)', color: 'var(--text1)', fontSize: 12,
        }}
      />
      <button
        onClick={handleApply}
        style={{
          padding: '5px 14px', borderRadius: 6, border: '1px solid var(--accent)',
          background: 'var(--accent)', color: '#fff', cursor: 'pointer', fontSize: 12, fontWeight: 500,
        }}
      >
        查询
      </button>
      {hasFilter && (
        <button
          onClick={handleClear}
          style={{
            padding: '5px 10px', borderRadius: 6, border: '1px solid var(--border)',
            background: 'transparent', color: 'var(--text2)', cursor: 'pointer', fontSize: 11,
          }}
        >
          清除筛选
        </button>
      )}
    </div>
  );
}
