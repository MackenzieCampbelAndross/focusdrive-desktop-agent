use tauri::State;
use serde::Serialize;
use crate::auth::{EmployeeProfile, OtpRequestResult, AuthState};
use crate::device::DeviceInfo;
use crate::errors::{AppError, Result};
use crate::session::{WorkSession, BreakSession, SessionSummary};
use crate::state::SessionState;
use crate::AppState;

#[derive(Serialize)]
pub struct AuthStateResponse {
    pub auth_state: AuthState,
    pub current_user: Option<EmployeeProfile>,
    pub pending_email: Option<String>,
    pub is_development_mode: bool,
}

#[derive(Serialize)]
pub struct SessionStateResponse {
    pub current_state: SessionState,
    pub is_recovered: bool,
    pub active_session: Option<WorkSession>,
    pub active_break: Option<BreakSession>,
    pub last_summary: Option<SessionSummary>,
}

#[derive(Serialize)]
pub struct AgentStatusResponse {
    pub agent_version: String,
    pub is_development_mode: bool,
    pub session_state: SessionState,
    pub authenticated_user: Option<String>,
    pub device_id: String,
    pub active_shift_id: Option<String>,
}

#[tauri::command]
pub async fn get_auth_state(state: State<'_, AppState>) -> Result<AuthStateResponse> {
    let auth = state.auth.lock().await;
    let session = state.session.lock().await;

    let auth_state = if session.current_state() == SessionState::RecoveryPendingAuth && auth.get_current_user().is_none() {
        AuthState::RecoveryPendingAuth
    } else if auth.get_current_user().is_some() {
        AuthState::Authenticated
    } else if auth.get_pending_email().is_some() {
        AuthState::OtpRequested
    } else {
        AuthState::LoggedOut
    };

    Ok(AuthStateResponse {
        auth_state,
        current_user: auth.get_current_user(),
        pending_email: auth.get_pending_email(),
        is_development_mode: auth.is_development_mode(),
    })
}

#[tauri::command]
pub async fn request_otp(email: String, state: State<'_, AppState>) -> Result<OtpRequestResult> {
    let auth = state.auth.lock().await;
    auth.request_otp(&email)
}

#[tauri::command]
pub async fn verify_otp(email: String, code: String, state: State<'_, AppState>) -> Result<EmployeeProfile> {
    let auth = state.auth.lock().await;
    let profile = auth.verify_otp(&email, &code)?;

    let mut session = state.session.lock().await;
    session.on_authenticated(&profile.id)?;

    Ok(profile)
}

#[tauri::command]
pub async fn logout(state: State<'_, AppState>) -> Result<()> {
    let mut session = state.session.lock().await;
    session.set_logged_out()?;

    let auth = state.auth.lock().await;
    auth.logout()?;

    Ok(())
}

#[tauri::command]
pub async fn get_session_state(state: State<'_, AppState>) -> Result<SessionStateResponse> {
    let session = state.session.lock().await;
    Ok(SessionStateResponse {
        current_state: session.current_state(),
        is_recovered: session.is_recovered(),
        active_session: session.active_session().cloned(),
        active_break: session.active_break().cloned(),
        last_summary: session.last_summary().cloned(),
    })
}

#[tauri::command]
pub async fn start_shift(state: State<'_, AppState>) -> Result<WorkSession> {
    let auth = state.auth.lock().await;
    let user = auth.get_current_user().ok_or_else(|| {
        AppError::AuthError("Must be authenticated with @purpledrive.com OTP to start a shift.".to_string())
    })?;

    let device = state.device_info.lock().await;

    let mut session = state.session.lock().await;
    session.start_shift(&user.id, &device.device_id)
}

#[tauri::command]
pub async fn start_manual_break(state: State<'_, AppState>) -> Result<BreakSession> {
    let mut session = state.session.lock().await;
    session.start_manual_break()
}

#[tauri::command]
pub async fn resume_work(state: State<'_, AppState>) -> Result<()> {
    let mut session = state.session.lock().await;
    session.resume_work()
}

#[tauri::command]
pub async fn end_shift(state: State<'_, AppState>) -> Result<SessionSummary> {
    let mut session = state.session.lock().await;
    session.end_shift()
}

#[tauri::command]
pub async fn dismiss_summary(state: State<'_, AppState>) -> Result<()> {
    let mut session = state.session.lock().await;
    session.dismiss_summary()
}

#[tauri::command]
pub async fn get_current_shift(state: State<'_, AppState>) -> Result<Option<WorkSession>> {
    let session = state.session.lock().await;
    Ok(session.active_session().cloned())
}

#[tauri::command]
pub async fn get_current_break(state: State<'_, AppState>) -> Result<Option<BreakSession>> {
    let session = state.session.lock().await;
    Ok(session.active_break().cloned())
}

#[tauri::command]
pub async fn get_device_info(state: State<'_, AppState>) -> Result<DeviceInfo> {
    let device = state.device_info.lock().await;
    Ok(device.clone())
}

#[tauri::command]
pub async fn get_agent_status(state: State<'_, AppState>) -> Result<AgentStatusResponse> {
    let auth = state.auth.lock().await;
    let session = state.session.lock().await;
    let device = state.device_info.lock().await;

    Ok(AgentStatusResponse {
        agent_version: env!("CARGO_PKG_VERSION").to_string(),
        is_development_mode: auth.is_development_mode(),
        session_state: session.current_state(),
        authenticated_user: auth.get_current_user().map(|u| u.email),
        device_id: device.device_id.clone(),
        active_shift_id: session.active_session().map(|s| s.id.clone()),
    })
}
