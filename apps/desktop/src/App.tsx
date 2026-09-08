import { useEffect, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  AuthStateResponse,
  DeviceInfo,
  EmployeeProfile,
  SessionStateResponse,
  WorkSession,
  BreakSession,
  SessionSummary
} from './types';
import { Header } from './components/Header';
import { LoginView } from './components/LoginView';
import { ShiftReadyView } from './components/ShiftReadyView';
import { WorkingView } from './components/WorkingView';
import { ManualBreakView } from './components/ManualBreakView';
import { ShiftSummaryView } from './components/ShiftSummaryView';

export default function App() {
  const [currentUser, setCurrentUser] = useState<EmployeeProfile | null>(null);
  const [isDevMode, setIsDevMode] = useState<boolean>(true);
  const [deviceInfo, setDeviceInfo] = useState<DeviceInfo | null>(null);
  const [sessionResponse, setSessionResponse] = useState<SessionStateResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  const syncState = useCallback(async () => {
    try {
      const authRes = await invoke<AuthStateResponse>('get_auth_state');
      setCurrentUser(authRes.current_user);
      setIsDevMode(authRes.is_development_mode);

      const devRes = await invoke<DeviceInfo>('get_device_info');
      setDeviceInfo(devRes);

      const sessRes = await invoke<SessionStateResponse>('get_session_state');
      setSessionResponse(sessRes);
    } catch (err) {
      console.error('Failed to sync state from Rust:', err);
    }
  }, []);

  useEffect(() => {
    syncState();
    const interval = setInterval(syncState, 2000);
    return () => clearInterval(interval);
  }, [syncState]);

  const handleLogin = async (email: string) => {
    setError(null);
    try {
      const profile = await invoke<EmployeeProfile>('login', { email, password: 'password' });
      setCurrentUser(profile);
      await syncState();
    } catch (err) {
      setError(String(err));
    }
  };

  const handleLogout = async () => {
    setError(null);
    try {
      await invoke('logout');
      setCurrentUser(null);
      await syncState();
    } catch (err) {
      setError(String(err));
    }
  };

  const handleStartShift = async () => {
    setError(null);
    try {
      await invoke<WorkSession>('start_shift');
      await syncState();
    } catch (err) {
      setError(String(err));
    }
  };

  const handleStartBreak = async () => {
    setError(null);
    try {
      await invoke<BreakSession>('start_manual_break');
      await syncState();
    } catch (err) {
      setError(String(err));
    }
  };

  const handleResumeWork = async () => {
    setError(null);
    try {
      await invoke('resume_work');
      await syncState();
    } catch (err) {
      setError(String(err));
    }
  };

  const handleEndShift = async () => {
    setError(null);
    try {
      await invoke<SessionSummary>('end_shift');
      await syncState();
    } catch (err) {
      setError(String(err));
    }
  };

  const handleDismissSummary = async () => {
    setError(null);
    try {
      await invoke('dismiss_summary');
      await syncState();
    } catch (err) {
      setError(String(err));
    }
  };

  const currentState = sessionResponse?.current_state ?? 'LOGGED_OUT';

  return (
    <div style={{ fontFamily: 'system-ui, -apple-system, sans-serif', maxWidth: '700px', margin: '0 auto', padding: '1.5rem' }}>
      <Header
        currentUser={currentUser}
        deviceInfo={deviceInfo}
        isDevMode={isDevMode}
        stateLabel={currentState}
      />

      {currentState === 'LOGGED_OUT' && (
        <LoginView onLogin={handleLogin} error={error} />
      )}

      {currentState === 'SHIFT_READY' && currentUser && (
        <ShiftReadyView
          currentUser={currentUser}
          onStartShift={handleStartShift}
          onLogout={handleLogout}
          error={error}
        />
      )}

      {currentState === 'WORKING' && sessionResponse?.active_session && (
        <WorkingView
          session={sessionResponse.active_session}
          isRecovered={sessionResponse.is_recovered}
          onStartBreak={handleStartBreak}
          onEndShift={handleEndShift}
          error={error}
        />
      )}

      {currentState === 'MANUAL_BREAK' && sessionResponse?.active_break && (
        <ManualBreakView
          breakSession={sessionResponse.active_break}
          onResumeWork={handleResumeWork}
          onEndShift={handleEndShift}
          error={error}
        />
      )}

      {currentState === 'SHIFT_SUMMARY' && sessionResponse?.last_summary && (
        <ShiftSummaryView
          summary={sessionResponse.last_summary}
          onDismiss={handleDismissSummary}
        />
      )}
    </div>
  );
}
