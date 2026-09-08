import { useState } from 'react';

interface LoginViewProps {
  onSendOtp: (email: string) => Promise<void>;
  onVerifyOtp: (code: string) => Promise<void>;
  onChangeEmail: () => void;
  pendingEmail: string | null;
  devOtpHint?: string | null;
  error?: string | null;
  isLoading: boolean;
}

export function LoginView({
  onSendOtp,
  onVerifyOtp,
  onChangeEmail,
  pendingEmail,
  devOtpHint,
  error,
  isLoading,
}: LoginViewProps) {
  const [email, setEmail] = useState(pendingEmail || '');
  const [otpCode, setOtpCode] = useState('');
  const [localValidationError, setLocalValidationError] = useState<string | null>(null);

  const handleSendOtp = async (e: React.FormEvent) => {
    e.preventDefault();
    setLocalValidationError(null);

    const trimmed = email.trim();
    if (!trimmed) {
      setLocalValidationError('Please enter your work email.');
      return;
    }

    const parts = trimmed.split('@');
    if (parts.length !== 2 || parts[1].toLowerCase() !== 'purpledrive.com') {
      setLocalValidationError('Only @purpledrive.com work email addresses are permitted.');
      return;
    }

    await onSendOtp(trimmed);
  };

  const handleVerifyOtp = async (e: React.FormEvent) => {
    e.preventDefault();
    setLocalValidationError(null);

    if (!otpCode.trim()) {
      setLocalValidationError('Please enter the verification code.');
      return;
    }

    await onVerifyOtp(otpCode.trim());
  };

  // Step 2: OTP Verification Screen
  if (pendingEmail) {
    return (
      <div style={{ padding: '2rem', border: '1px solid #e2e8f0', borderRadius: '10px', backgroundColor: '#ffffff', boxShadow: '0 1px 3px rgba(0,0,0,0.05)' }}>
        <div style={{ display: 'inline-block', padding: '4px 10px', borderRadius: '12px', backgroundColor: '#ede9fe', color: '#6b21a8', fontSize: '0.75rem', fontWeight: 700, marginBottom: '0.75rem' }}>
          PURPLEDRIVE AUTHENTICATION
        </div>
        <h2 style={{ marginTop: 0, marginBottom: '0.5rem', color: '#1e293b' }}>Enter Verification Code</h2>
        <p style={{ color: '#64748b', fontSize: '0.9rem', marginBottom: '1.5rem', lineHeight: 1.4 }}>
          Enter the verification code sent to <strong>{pendingEmail}</strong>.
        </p>

        {devOtpHint && (
          <div style={{ padding: '0.75rem 1rem', backgroundColor: '#f0fdf4', border: '1px solid #bbf7d0', color: '#166534', borderRadius: '8px', marginBottom: '1rem', fontSize: '0.85rem' }}>
            <strong>Local Development Mode:</strong> Verification code is <code>{devOtpHint}</code>
          </div>
        )}

        {(error || localValidationError) && (
          <div style={{ padding: '0.75rem 1rem', backgroundColor: '#fef2f2', border: '1px solid #fecaca', color: '#991b1b', borderRadius: '8px', marginBottom: '1rem', fontSize: '0.85rem' }}>
            {localValidationError || error}
          </div>
        )}

        <form onSubmit={handleVerifyOtp}>
          <div style={{ marginBottom: '1.5rem' }}>
            <label style={{ display: 'block', fontSize: '0.875rem', fontWeight: 600, marginBottom: '0.5rem', color: '#334155' }}>
              6-Digit Verification Code
            </label>
            <input
              type="text"
              inputMode="numeric"
              placeholder="e.g. 123456"
              value={otpCode}
              onChange={(e) => setOtpCode(e.target.value)}
              maxLength={10}
              autoFocus
              style={{
                width: '100%',
                padding: '0.75rem',
                borderRadius: '8px',
                border: '1px solid #cbd5e1',
                fontSize: '1.25rem',
                fontFamily: 'monospace',
                letterSpacing: '0.2em',
                textAlign: 'center',
                boxSizing: 'border-box'
              }}
              required
            />
          </div>

          <button
            type="submit"
            disabled={isLoading}
            style={{
              width: '100%',
              padding: '0.85rem',
              backgroundColor: '#6b21a8',
              color: '#ffffff',
              border: 'none',
              borderRadius: '8px',
              fontWeight: 700,
              fontSize: '1rem',
              cursor: isLoading ? 'not-allowed' : 'pointer',
              opacity: isLoading ? 0.7 : 1,
              marginBottom: '1rem',
            }}
          >
            {isLoading ? 'VERIFYING...' : 'VERIFY OTP'}
          </button>

          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <button
              type="button"
              onClick={() => onSendOtp(pendingEmail)}
              disabled={isLoading}
              style={{
                background: 'none',
                border: 'none',
                color: '#6b21a8',
                fontSize: '0.85rem',
                fontWeight: 600,
                cursor: 'pointer',
                textDecoration: 'underline'
              }}
            >
              Resend Code
            </button>

            <button
              type="button"
              onClick={onChangeEmail}
              disabled={isLoading}
              style={{
                background: 'none',
                border: 'none',
                color: '#64748b',
                fontSize: '0.85rem',
                cursor: 'pointer'
              }}
            >
              Change Email
            </button>
          </div>
        </form>
      </div>
    );
  }

  // Step 1: Request OTP Screen
  return (
    <div style={{ padding: '2rem', border: '1px solid #e2e8f0', borderRadius: '10px', backgroundColor: '#ffffff', boxShadow: '0 1px 3px rgba(0,0,0,0.05)' }}>
      <div style={{ display: 'inline-block', padding: '4px 10px', borderRadius: '12px', backgroundColor: '#ede9fe', color: '#6b21a8', fontSize: '0.75rem', fontWeight: 700, marginBottom: '0.75rem' }}>
        PURPLEDRIVE WORKSTATION
      </div>
      <h2 style={{ marginTop: 0, marginBottom: '0.5rem', color: '#1e293b' }}>Sign in to FocusDrive</h2>
      <p style={{ color: '#64748b', fontSize: '0.9rem', marginBottom: '1.5rem', lineHeight: 1.4 }}>
        Sign in with your <strong>PurpleDrive</strong> work email to access your desktop telemetry companion.
      </p>

      {(error || localValidationError) && (
        <div style={{ padding: '0.75rem 1rem', backgroundColor: '#fef2f2', border: '1px solid #fecaca', color: '#991b1b', borderRadius: '8px', marginBottom: '1rem', fontSize: '0.85rem' }}>
          {localValidationError || error}
        </div>
      )}

      <form onSubmit={handleSendOtp}>
        <div style={{ marginBottom: '1.5rem' }}>
          <label style={{ display: 'block', fontSize: '0.875rem', fontWeight: 600, marginBottom: '0.5rem', color: '#334155' }}>
            Work Email Address
          </label>
          <input
            type="email"
            placeholder="name@purpledrive.com"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            style={{
              width: '100%',
              padding: '0.75rem',
              borderRadius: '8px',
              border: '1px solid #cbd5e1',
              fontSize: '0.95rem',
              boxSizing: 'border-box'
            }}
            required
            autoFocus
          />
          <div style={{ fontSize: '0.75rem', color: '#94a3b8', marginTop: '0.35rem' }}>
            Only @purpledrive.com accounts are authorized.
          </div>
        </div>

        <button
          type="submit"
          disabled={isLoading}
          style={{
            width: '100%',
            padding: '0.85rem',
            backgroundColor: '#6b21a8',
            color: '#ffffff',
            border: 'none',
            borderRadius: '8px',
            fontWeight: 700,
            fontSize: '1rem',
            cursor: isLoading ? 'not-allowed' : 'pointer',
            opacity: isLoading ? 0.7 : 1
          }}
        >
          {isLoading ? 'SENDING CODE...' : 'SEND OTP'}
        </button>
      </form>
    </div>
  );
}
