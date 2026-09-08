import { useEffect, useState } from 'react';
import { BreakSession } from '../types';
import { formatSeconds } from '../utils';

interface ManualBreakViewProps {
  breakSession: BreakSession;
  onResumeWork: () => void;
  onEndShift: () => void;
  error?: string | null;
}

export function ManualBreakView({ breakSession, onResumeWork, onEndShift, error }: ManualBreakViewProps) {
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
    <div style={{ padding: '1.5rem', border: '1px solid #fef08a', borderRadius: '8px', backgroundColor: '#fefce8' }}>
      {error && (
        <div style={{ padding: '0.75rem', backgroundColor: '#fef2f2', border: '1px solid #fecaca', color: '#991b1b', borderRadius: '6px', marginBottom: '1rem', fontSize: '0.875rem' }}>
          {error}
        </div>
      )}

      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
        <div>
          <span style={{ backgroundColor: '#eab308', color: '#713f12', padding: '3px 10px', borderRadius: '12px', fontSize: '0.8rem', fontWeight: 700 }}>
            ☕ ON MANUAL BREAK
          </span>
          <h2 style={{ margin: '0.5rem 0 0 0', color: '#854d0e' }}>Manual Break</h2>
        </div>
        <div style={{ textAlign: 'right' }}>
          <div style={{ fontSize: '0.75rem', color: '#a16207', fontWeight: 600 }}>BREAK ELAPSED</div>
          <div style={{ fontSize: '2rem', fontWeight: 800, fontFamily: 'monospace', color: '#854d0e' }}>
            {formatSeconds(elapsedSeconds)}
          </div>
        </div>
      </div>

      <div style={{ padding: '0.75rem 1rem', backgroundColor: '#ffffff', borderRadius: '6px', border: '1px solid #fef08a', marginBottom: '1.5rem', fontSize: '0.85rem', color: '#854d0e' }}>
        Break Started: {new Date(breakSession.started_at).toLocaleTimeString()} | Telemetry collection paused.
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
