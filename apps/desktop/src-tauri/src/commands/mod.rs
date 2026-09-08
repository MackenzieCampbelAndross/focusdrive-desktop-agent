use tauri::State;
use serde::Serialize;
use crate::auth::EmployeeProfile;
use crate::device::DeviceInfo;
use crate::errors::{AppError, Result};
use crate::session::{WorkSession, BreakSession, SessionSummary};
use crate::state::SessionState;
use crate::AppState;

#[derive(Serialize)]
pub struct AuthStateResponse {
    pub current_user: Option<EmployeeProfile>,
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
pub fn get_auth_state(state: State<'_, AppState>) -> Result<AuthStateResponse> {
    let auth = state.auth.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(AuthStateResponse {
        current_user: auth.get_current_user(),
        is_development_mode: auth.is_development_mode(),
    })
}

#[tauri::command]
pub fn login(email: String, password: String, state: State<'_, AppState>) -> Result<EmployeeProfile> {
    let auth = state.auth.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
    let profile = auth.login(&email, &password)?;
    
    let mut session = state.session.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
    session.set_logged_in()?;
    
    Ok(profile)
}

#[tauri::command]
pub fn logout(state: State<'_, AppState>) -> Result<()> {
    let mut session = state.session.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
    session.set_logged_out()?;
    
    let auth = state.auth.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
    auth.logout()?;
    
    Ok(())
}

#[tauri::command]
pub fn get_session_state(state: State<'_, AppState>) -> Result<SessionStateResponse> {
    let session = state.session.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(SessionStateResponse {
        current_state: session.current_state(),
        is_recovered: session.is_recovered(),
        active_session: session.active_session().cloned(),
        active_break: session.active_break().cloned(),
        last_summary: session.last_summary().cloned(),
    })
}

#[tauri::command]
pub fn start_shift(state: State<'_, AppState>) -> Result<WorkSession> {
    let auth = state.auth.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
    let user = auth.get_current_user().ok_or_else(|| {
        AppError::AuthError("Must be logged in to start a shift.".to_string())
    })?;

    let device = state.device_info.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
    
    let mut session = state.session.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
    session.start_shift(&user.id, &device.device_id)
}

#[tauri::command]
pub fn start_manual_break(state: State<'_, AppState>) -> Result<BreakSession> {
    let mut session = state.session.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
    session.start_manual_break()
}

#[tauri::command]
pub fn resume_work(state: State<'_, AppState>) -> Result<()> {
    let mut session = state.session.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
    session.resume_work()
}

#[tauri::command]
pub fn end_shift(state: State<'_, AppState>) -> Result<SessionSummary> {
    let mut session = state.session.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
    session.end_shift()
}

#[tauri::command]
pub fn dismiss_summary(state: State<'_, AppState>) -> Result<()> {
    let mut session = state.session.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
    session.dismiss_summary()
}

#[tauri::command]
pub fn get_current_shift(state: State<'_, AppState>) -> Result<Option<WorkSession>> {
    let session = state.session.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(session.active_session().cloned())
}

#[tauri::command]
pub fn get_current_break(state: State<'_, AppState>) -> Result<Option<BreakSession>> {
    let session = state.session.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(session.active_break().cloned())
}

#[tauri::command]
pub fn get_device_info(state: State<'_, AppState>) -> Result<DeviceInfo> {
    let device = state.device_info.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(device.clone())
}

#[tauri::command]
pub fn get_agent_status(state: State<'_, AppState>) -> Result<AgentStatusResponse> {
    let auth = state.auth.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
    let session = state.session.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
    let device = state.device_info.lock().map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(AgentStatusResponse {
        agent_version: env!("CARGO_PKG_VERSION").to_string(),
        is_development_mode: auth.is_development_mode(),
        session_state: session.current_state(),
        authenticated_user: auth.get_current_user().map(|u| u.email),
        device_id: device.device_id.clone(),
        active_shift_id: session.active_session().map(|s| s.id.clone()),
    })
}
