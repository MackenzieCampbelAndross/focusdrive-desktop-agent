import { useEffect, useState } from 'react';
import { WorkSession } from '../types';
import { formatSeconds } from '../utils';

interface WorkingViewProps {
  session: WorkSession;
  isRecovered: boolean;
  onStartBreak: () => void;
  onEndShift: () => void;
  error?: string | null;
}

export function WorkingView({ session, isRecovered, onStartBreak, onEndShift, error }: WorkingViewProps) {
  const [elapsedSeconds, setElapsedSeconds] = useState<number>(0);

  useEffect(() => {
    const startTime = new Date(session.started_at).getTime();

    const updateTimer = () => {
      const now = new Date().getTime();
      const diff = Math.max(0, Math.floor((now - startTime) / 1000));
      setElapsedSeconds(diff);
    };

    updateTimer();
    const interval = setInterval(updateTimer, 1000);
    return () => clearInterval(interval);
  }, [session.started_at]);

  return (
    <div style={{ padding: '1.5rem', border: '1px solid #bbf7d0', borderRadius: '8px', backgroundColor: '#f0fdf4' }}>
      {isRecovered && (
        <div style={{ padding: '0.75rem', backgroundColor: '#fef3c7', border: '1px solid #fde047', color: '#854d0e', borderRadius: '6px', marginBottom: '1rem', fontSize: '0.85rem' }}>
          <strong>Shift Restored:</strong> Restored active session started at {new Date(session.started_at).toLocaleTimeString()}.
        </div>
      )}

      {error && (
        <div style={{ padding: '0.75rem', backgroundColor: '#fef2f2', border: '1px solid #fecaca', color: '#991b1b', borderRadius: '6px', marginBottom: '1rem', fontSize: '0.875rem' }}>
          {error}
        </div>
      )}

      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
        <div>
          <span style={{ backgroundColor: '#22c55e', color: '#ffffff', padding: '3px 10px', borderRadius: '12px', fontSize: '0.8rem', fontWeight: 700 }}>
            ● SHIFT ACTIVE
          </span>
          <h2 style={{ margin: '0.5rem 0 0 0', color: '#14532d' }}>Working</h2>
        </div>
        <div style={{ textAlign: 'right' }}>
          <div style={{ fontSize: '0.75rem', color: '#15803d', fontWeight: 600 }}>SHIFT ELAPSED</div>
          <div style={{ fontSize: '2rem', fontWeight: 800, fontFamily: 'monospace', color: '#166534' }}>
            {formatSeconds(elapsedSeconds)}
          </div>
        </div>
      </div>

      <div style={{ padding: '0.75rem 1rem', backgroundColor: '#ffffff', borderRadius: '6px', border: '1px solid #dcfce7', marginBottom: '1.5rem', fontSize: '0.85rem', color: '#166534' }}>
        Started: {new Date(session.started_at).toLocaleTimeString()} | Session ID: {session.id.substring(0, 8)}...
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem' }}>
        <button
          onClick={onStartBreak}
          style={{
            padding: '0.85rem',
            backgroundColor: '#eab308',
            color: '#713f12',
            border: 'none',
            borderRadius: '8px',
            fontWeight: 700,
            fontSize: '1rem',
            cursor: 'pointer'
          }}
        >
          ☕ START BREAK
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
