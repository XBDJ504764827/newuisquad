'use client';

import { useState } from 'react';

export function ActionBtn({ children, onClick, color, bg }: { children: string; onClick: () => void; color: string; bg: string }) {
  const [hover, setHover] = useState(false);
  return (
    <span
      onClick={onClick}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      style={{
        cursor: 'pointer', fontSize: 9, fontWeight: 600,
        padding: '3px 7px', borderRadius: 4,
        background: hover ? color : bg,
        color: hover ? '#fff' : color,
        transition: 'all .12s', whiteSpace: 'nowrap',
        border: `1px solid ${hover ? color : 'transparent'}`,
      }}
    >{children}</span>
  );
}
