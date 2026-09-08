import { useEffect, useState } from 'react';
import { BreakSession } from '../types';
import { formatSeconds } from '../utils';

interface AutomaticBreakViewProps {
  breakSession: BreakSession;
  onResumeWork: () => void;
  onEndShift: () => void;
  error?: string | null;
}

export function AutomaticBreakView({ breakSession, onResumeWork, onEndShift, error }: AutomaticBreakViewProps) {
  const [elapsedSeconds, setElapsedSeconds] = useState<number>(0);

  useEffect(() => {
    const startTime = new Date(breakSession.started_at).getTime();

    const updateTimer = () => {
      const now = new Date().getTime();
      const diff = Math.max(0, Math.floor((now - startTime) / 1000));
      setElapsedSeconds(diff);
    };

    updateTimer();
    const interval = setInterval(updateTimer, 1000);
    return () => clearInterval(interval);
  }, [breakSession.started_at]);

  return (
    <div style={{ padding: '1.5rem', border: '1px solid #fed7aa', borderRadius: '8px', backgroundColor: '#fff7ed' }}>
      {error && (
        <div style={{ padding: '0.75rem', backgroundColor: '#fef2f2', border: '1px solid #fecaca', color: '#991b1b', borderRadius: '6px', marginBottom: '1rem', fontSize: '0.875rem' }}>
          {error}
        </div>
      )}

      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
        <div>
          <span style={{ backgroundColor: '#f97316', color: '#ffffff', padding: '3px 10px', borderRadius: '12px', fontSize: '0.8rem', fontWeight: 700 }}>
            💤 AUTOMATIC IDLE BREAK
          </span>
          <h2 style={{ margin: '0.5rem 0 0 0', color: '#9a3412' }}>Idle Break Active</h2>
        </div>
        <div style={{ textAlign: 'right' }}>
          <div style={{ fontSize: '0.75rem', color: '#c2410c', fontWeight: 600 }}>IDLE DURATION</div>
          <div style={{ fontSize: '2rem', fontWeight: 800, fontFamily: 'monospace', color: '#9a3412' }}>
            {formatSeconds(elapsedSeconds)}
          </div>
        </div>
      </div>

      <div style={{ padding: '0.75rem 1rem', backgroundColor: '#ffffff', borderRadius: '6px', border: '1px solid #fed7aa', marginBottom: '1.5rem', fontSize: '0.85rem', color: '#9a3412' }}>
        System detected inactive keyboard & mouse for &gt;60 seconds. Move mouse or press any key to resume automatically, or click below.
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem' }}>
        <button
          onClick={onResumeWork}
          style={{
            padding: '0.85rem',
            backgroundColor: '#16a34a',
            color: '#ffffff',
            border: 'none',
            borderRadius: '8px',
            fontWeight: 700,
            fontSize: '1rem',
            cursor: 'pointer'
          }}
        >
          ▶ RESUME WORK
        </button>

        <button
          onClick={onEndShift}
          style={{
            padding: '0.85rem',
            backgroundColor: '#dc2626',
            color: '#ffffff',
            border: 'none',
            borderRadius: '8px',
            fontWeight: 700,
            fontSize: '1rem',
            cursor: 'pointer'
          }}
        >
          ⏹ END SHIFT
        </button>
      </div>
    </div>
  );
}
