import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { DEFAULT_APP_NAME } from '@focusdrive/config';

export default function App() {
  const [status, setStatus] = useState<string>('Initializing...');
  const [version, setVersion] = useState<string>('');

  useEffect(() => {
    invoke<string>('health_check')
      .then((res) => {
        setStatus(res);
      })
      .catch((err) => {
        setStatus(`Error: ${String(err)}`);
      });

    invoke<string>('get_agent_version')
      .then((ver) => setVersion(ver))
      .catch(() => setVersion('0.1.0'));
  }, []);

  return (
    <div style={{ fontFamily: 'sans-serif', padding: '2rem', maxWidth: '600px', margin: '0 auto' }}>
      <h1>{DEFAULT_APP_NAME}</h1>
      <div style={{ padding: '1rem', border: '1px solid #ccc', borderRadius: '8px', background: '#f9f9f9' }}>
        <p><strong>System Status:</strong> {status}</p>
        <p><strong>Agent Version:</strong> {version || '0.1.0'}</p>
        <p><strong>State:</strong> LOGGED_OUT (Phase 0 Foundation)</p>
      </div>
      <p style={{ marginTop: '1.5rem', fontSize: '0.875rem', color: '#666' }}>
        FocusDrive Desktop Agent telemetry is stopped until shift start.
      </p>
    </div>
  );
}
