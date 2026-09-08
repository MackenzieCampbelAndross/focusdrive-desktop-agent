export type SessionState =
  | 'LOGGED_OUT'
  | 'SHIFT_READY'
  | 'WORKING'
  | 'MANUAL_BREAK'
  | 'AUTOMATIC_BREAK'
  | 'SHIFT_COMPLETING'
  | 'SHIFT_SUMMARY'
  | 'RECOVERY_PENDING_AUTH';

export type AuthState =
  | 'AUTH_LOADING'
  | 'LOGGED_OUT'
  | 'OTP_REQUESTED'
  | 'VERIFYING_OTP'
  | 'AUTHENTICATED'
  | 'RECOVERY_PENDING_AUTH';

export interface EmployeeProfile {
  id: string;
  name: string;
  email: string;
  role: string;
  department?: string;
}

export interface OtpRequestResult {
  email: string;
  message: string;
  dev_otp?: string;
}

export interface WorkSession {
  id: string;
  employee_id: string;
  device_id: string;
  started_at: string;
  ended_at?: string;
  status: string;
  created_at: string;
}

export interface BreakSession {
  id: string;
  work_session_id: string;
  employee_id: string;
  break_type: string;
  started_at: string;
  ended_at?: string;
  duration_seconds: number;
  created_at: string;
}

export interface SessionSummary {
  work_session_id: string;
  employee_id: string;
  started_at: string;
  ended_at: string;
  total_shift_duration_seconds: number;
  manual_break_duration_seconds: number;
  effective_working_duration_seconds: number;
}

export interface DeviceInfo {
  device_id: string;
  device_name: string;
  operating_system: string;
  agent_version: string;
  created_at: string;
  last_seen_at: string;
}

export interface AuthStateResponse {
  auth_state: AuthState;
  current_user: EmployeeProfile | null;
  pending_email: string | null;
  is_development_mode: boolean;
}

export interface SessionStateResponse {
  current_state: SessionState;
  is_recovered: boolean;
  active_session: WorkSession | null;
  active_break: BreakSession | null;
  last_summary: SessionSummary | null;
}
