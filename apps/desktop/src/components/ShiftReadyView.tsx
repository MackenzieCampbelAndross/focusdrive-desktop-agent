import { EmployeeProfile } from '../types';

interface ShiftReadyViewProps {
  currentUser: EmployeeProfile;
  onStartShift: () => void;
  onLogout: () => void;
  error?: string | null;
}

export function ShiftReadyView({ currentUser, onStartShift, onLogout, error }: ShiftReadyViewProps) {
  return (
    <div style={{ padding: '1.5rem', border: '1px solid #cbd5e1', borderRadius: '8px', backgroundColor: '#ffffff' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
        <div>
          <h2 style={{ margin: 0, color: '#0f172a' }}>Shift Ready</h2>
          <div style={{ color: '#64748b', fontSize: '0.9rem' }}>
            Workstation active for <strong>{currentUser.name}</strong> ({currentUser.role})
          </div>
        </div>
        <button
          onClick={onLogout}
          style={{
            padding: '0.4rem 0.8rem',
            backgroundColor: '#ef4444',
            color: '#ffffff',
            border: 'none',
            borderRadius: '6px',
            cursor: 'pointer',
            fontSize: '0.85rem',
            fontWeight: 600
          }}
        >
          LOG OUT
        </button>
      </div>

      {error && (
        <div style={{ padding: '0.75rem', backgroundColor: '#fef2f2', border: '1px solid #fecaca', color: '#991b1b', borderRadius: '6px', marginBottom: '1rem', fontSize: '0.875rem' }}>
          {error}
        </div>
      )}

      <div style={{
        padding: '1.25rem',
        backgroundColor: '#f8fafc',
        border: '1px solid #e2e8f0',
        borderRadius: '8px',
        textAlign: 'center',
        marginBottom: '1.5rem'
      }}>
        <div style={{ fontSize: '1.1rem', fontWeight: 600, color: '#334155', marginBottom: '0.5rem' }}>
          Telemetry is currently STOPPED
        </div>
        <p style={{ color: '#64748b', fontSize: '0.875rem', margin: 0 }}>
          Work session telemetry will only begin when you explicitly click <strong>START SHIFT</strong> below.
        </p>
      </div>

      <button
        onClick={onStartShift}
        style={{
          width: '100%',
          padding: '1rem',
          backgroundColor: '#16a34a',
          color: '#ffffff',
          border: 'none',
          borderRadius: '8px',
          fontWeight: 700,
          fontSize: '1.15rem',
          cursor: 'pointer',
          boxShadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1)'
        }}
      >
        ▶ START SHIFT
      </button>
    </div>
  );
}
