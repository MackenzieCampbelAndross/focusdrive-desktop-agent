import { useState } from 'react';

interface LoginViewProps {
  onLogin: (email: string) => void;
  error?: string | null;
}

export function LoginView({ onLogin, error }: LoginViewProps) {
  const [email, setEmail] = useState('alex@focusdrive.dev');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onLogin(email);
  };

  return (
    <div style={{ padding: '1.5rem', border: '1px solid #e2e8f0', borderRadius: '8px', backgroundColor: '#ffffff' }}>
      <h2 style={{ marginTop: 0 }}>Employee Workstation Login</h2>
      <p style={{ color: '#64748b', fontSize: '0.9rem' }}>
        Log in to access your FocusDrive desktop agent telemetry companion.
      </p>

      {error && (
        <div style={{ padding: '0.75rem', backgroundColor: '#fef2f2', border: '1px solid #fecaca', color: '#991b1b', borderRadius: '6px', marginBottom: '1rem', fontSize: '0.875rem' }}>
          {error}
        </div>
      )}

      <form onSubmit={handleSubmit}>
        <div style={{ marginBottom: '1rem' }}>
          <label style={{ display: 'block', fontSize: '0.875rem', fontWeight: 600, marginBottom: '0.25rem' }}>
            Work Email Address
          </label>
          <input
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            style={{ width: '100%', padding: '0.5rem', borderRadius: '6px', border: '1px solid #cbd5e1', fontSize: '0.9rem' }}
            required
          />
        </div>

        <div style={{ marginBottom: '1.5rem' }}>
          <label style={{ display: 'block', fontSize: '0.875rem', fontWeight: 600, marginBottom: '0.25rem' }}>
            Password
          </label>
          <input
            type="password"
            value="••••••••••••"
            disabled
            style={{ width: '100%', padding: '0.5rem', borderRadius: '6px', border: '1px solid #cbd5e1', backgroundColor: '#f8fafc', fontSize: '0.9rem' }}
          />
          <span style={{ fontSize: '0.75rem', color: '#94a3b8' }}>Password auto-accepted in Development Auth Mode.</span>
        </div>

        <button
          type="submit"
          style={{
            width: '100%',
            padding: '0.75rem',
            backgroundColor: '#2563eb',
            color: '#ffffff',
            border: 'none',
            borderRadius: '6px',
            fontWeight: 600,
            fontSize: '1rem',
            cursor: 'pointer'
          }}
        >
          LOG IN TO WORKSTATION
        </button>
      </form>
    </div>
  );
}
