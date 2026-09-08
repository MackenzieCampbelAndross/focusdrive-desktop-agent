import { SessionSummary } from '../types';
import { formatDurationLabel } from '../utils';

interface ShiftSummaryViewProps {
  summary: SessionSummary;
  onDismiss: () => void;
}

export function ShiftSummaryView({ summary, onDismiss }: ShiftSummaryViewProps) {
  return (
    <div style={{ padding: '1.5rem', border: '1px solid #cbd5e1', borderRadius: '8px', backgroundColor: '#ffffff' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1.5rem' }}>
        <div>
          <span style={{ backgroundColor: '#64748b', color: '#ffffff', padding: '3px 10px', borderRadius: '12px', fontSize: '0.8rem', fontWeight: 700 }}>
            ✓ SHIFT COMPLETED
          </span>
          <h2 style={{ margin: '0.5rem 0 0 0', color: '#0f172a' }}>Shift Summary</h2>
        </div>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem', marginBottom: '1.5rem' }}>
        <div style={{ padding: '1rem', backgroundColor: '#f8fafc', borderRadius: '8px', border: '1px solid #e2e8f0' }}>
          <div style={{ fontSize: '0.75rem', color: '#64748b', fontWeight: 600 }}>SHIFT START</div>
          <div style={{ fontSize: '1rem', fontWeight: 700, color: '#0f172a', marginTop: '0.25rem' }}>
            {new Date(summary.started_at).toLocaleTimeString()}
          </div>
        </div>

        <div style={{ padding: '1rem', backgroundColor: '#f8fafc', borderRadius: '8px', border: '1px solid #e2e8f0' }}>
          <div style={{ fontSize: '0.75rem', color: '#64748b', fontWeight: 600 }}>SHIFT END</div>
          <div style={{ fontSize: '1rem', fontWeight: 700, color: '#0f172a', marginTop: '0.25rem' }}>
            {new Date(summary.ended_at).toLocaleTimeString()}
          </div>
        </div>
      </div>

      <div style={{ padding: '1.25rem', backgroundColor: '#f1f5f9', borderRadius: '8px', border: '1px solid #cbd5e1', marginBottom: '1.5rem' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.75rem' }}>
          <span style={{ color: '#475569', fontSize: '0.9rem' }}>Total Shift Duration:</span>
          <strong style={{ color: '#0f172a' }}>{formatDurationLabel(summary.total_shift_duration_seconds)}</strong>
        </div>

        <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.75rem' }}>
          <span style={{ color: '#475569', fontSize: '0.9rem' }}>Manual Break Duration:</span>
          <strong style={{ color: '#d97706' }}>{formatDurationLabel(summary.manual_break_duration_seconds)}</strong>
        </div>

        <div style={{ height: '1px', backgroundColor: '#cbd5e1', margin: '0.75rem 0' }} />

        <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '1.05rem' }}>
          <span style={{ color: '#0f172a', fontWeight: 700 }}>Effective Working Duration:</span>
          <strong style={{ color: '#16a34a', fontWeight: 800 }}>
            {formatDurationLabel(summary.effective_working_duration_seconds)}
          </strong>
        </div>
      </div>

      <button
        onClick={onDismiss}
        style={{
          width: '100%',
          padding: '0.85rem',
          backgroundColor: '#2563eb',
          color: '#ffffff',
          border: 'none',
          borderRadius: '8px',
          fontWeight: 700,
          fontSize: '1rem',
          cursor: 'pointer'
        }}
      >
        RETURN TO SHIFT READY
      </button>
    </div>
  );
}
