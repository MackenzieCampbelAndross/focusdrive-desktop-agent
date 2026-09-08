import { useEffect, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  AuthStateResponse,
  DeviceInfo,
  EmployeeProfile,
  SessionStateResponse,
  WorkSession,
  BreakSession,
  SessionSummary,
  OtpRequestResult,
} from './types';
import { Header } from './components/Header';
import { LoginView } from './components/LoginView';
import { ShiftReadyView } from './components/ShiftReadyView';
import { WorkingView } from './components/WorkingView';
import { ManualBreakView } from './components/ManualBreakView';
import { AutomaticBreakView } from './components/AutomaticBreakView';
import { ShiftSummaryView } from './components/ShiftSummaryView';

export default function App() {
  const [currentUser, setCurrentUser] = useState<EmployeeProfile | null>(null);
  const [isDevMode, setIsDevMode] = useState<boolean>(true);
  const [pendingEmail, setPendingEmail] = useState<string | null>(null);
  const [devOtpHint, setDevOtpHint] = useState<string | null>(null);
  const [deviceInfo, setDeviceInfo] = useState<DeviceInfo | null>(null);
  const [sessionResponse, setSessionResponse] = useState<SessionStateResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(false);

  const syncState = useCallback(async () => {
    try {
      const authRes = await invoke<AuthStateResponse>('get_auth_state');
      setCurrentUser(authRes.current_user);
      setIsDevMode(authRes.is_development_mode);
      if (!currentUser && authRes.pending_email) {
        setPendingEmail(authRes.pending_email);
      }

      const devRes = await invoke<DeviceInfo>('get_device_info');
      setDeviceInfo(devRes);

      const sessRes = await invoke<SessionStateResponse>('get_session_state');
      setSessionResponse(sessRes);
    } catch (err) {
      console.error('Failed to sync state from Rust:', err);
    }
  }, [currentUser]);

  useEffect(() => {
    syncState();
    const interval = setInterval(syncState, 2000);
    return () => clearInterval(interval);
  }, [syncState]);

  const handleSendOtp = async (email: string) => {
    setError(null);
    setIsLoading(true);
    try {
      const result = await invoke<OtpRequestResult>('request_otp', { email });
      setPendingEmail(result.email);
      if (result.dev_otp) {
        setDevOtpHint(result.dev_otp);
      } else {
        setDevOtpHint(null);
      }
      await syncState();
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  };

  const handleVerifyOtp = async (code: string) => {
    if (!pendingEmail) return;
    setError(null);
    setIsLoading(true);
    try {
      const profile = await invoke<EmployeeProfile>('verify_otp', { email: pendingEmail, code });
      setCurrentUser(profile);
      setPendingEmail(null);
      setDevOtpHint(null);
      await syncState();
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  };

  const handleChangeEmail = () => {
    setPendingEmail(null);
    setDevOtpHint(null);
    setError(null);
  };

  const handleLogout = async () => {
    setError(null);
    try {
      await invoke('logout');
      setCurrentUser(null);
      setPendingEmail(null);
      setDevOtpHint(null);
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

  const currentSessionState = sessionResponse?.current_state ?? 'LOGGED_OUT';
  const isPendingRecovery = currentSessionState === 'RECOVERY_PENDING_AUTH';

  return (
    <div style={{ fontFamily: 'system-ui, -apple-system, sans-serif', maxWidth: '720px', margin: '0 auto', padding: '1.5rem' }}>
      <Header
        currentUser={currentUser}
        deviceInfo={deviceInfo}
        isDevMode={isDevMode}
        stateLabel={currentSessionState}
      />

      {isPendingRecovery && !currentUser && (
        <div style={{ padding: '0.85rem 1.25rem', backgroundColor: '#eff6ff', border: '1px solid #bfdbfe', borderRadius: '8px', color: '#1e40af', marginBottom: '1.25rem', fontSize: '0.9rem' }}>
          <strong>Shift Recovery Available:</strong> An unfinished work session was recovered from your workstation. Sign in with your matching @purpledrive.com email to restore access.
        </div>
      )}

      {(!currentUser || currentSessionState === 'LOGGED_OUT' || isPendingRecovery) && (
        <LoginView
          onSendOtp={handleSendOtp}
          onVerifyOtp={handleVerifyOtp}
          onChangeEmail={handleChangeEmail}
          pendingEmail={pendingEmail}
          devOtpHint={devOtpHint}
          error={error}
          isLoading={isLoading}
        />
      )}

      {currentSessionState === 'SHIFT_READY' && currentUser && (
        <ShiftReadyView
          currentUser={currentUser}
          onStartShift={handleStartShift}
          onLogout={handleLogout}
          error={error}
        />
      )}

      {currentSessionState === 'WORKING' && sessionResponse?.active_session && (
        <WorkingView
          session={sessionResponse.active_session}
          isRecovered={sessionResponse.is_recovered}
          onStartBreak={handleStartBreak}
          onEndShift={handleEndShift}
          error={error}
        />
      )}

      {currentSessionState === 'MANUAL_BREAK' && sessionResponse?.active_break && (
        <ManualBreakView
          breakSession={sessionResponse.active_break}
          onResumeWork={handleResumeWork}
          onEndShift={handleEndShift}
          error={error}
        />
      )}

      {currentSessionState === 'AUTOMATIC_BREAK' && sessionResponse?.active_break && (
        <AutomaticBreakView
          breakSession={sessionResponse.active_break}
          onResumeWork={handleResumeWork}
          onEndShift={handleEndShift}
          error={error}
        />
      )}

      {currentSessionState === 'SHIFT_SUMMARY' && sessionResponse?.last_summary && (
        <ShiftSummaryView
          summary={sessionResponse.last_summary}
          onDismiss={handleDismissSummary}
        />
      )}
    </div>
  );
}
