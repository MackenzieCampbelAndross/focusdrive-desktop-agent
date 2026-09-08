pub mod dev_adapter;
pub mod supabase_adapter;
pub mod domain_validator;

use serde::{Deserialize, Serialize};
use crate::errors::{AppError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMode {
    Development,
    SupabaseProduction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthState {
    AuthLoading,
    LoggedOut,
    OtpRequested,
    VerifyingOtp,
    Authenticated,
    RecoveryPendingAuth,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmployeeProfile {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub department: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtpRequestResult {
    pub email: String,
    pub message: String,
    pub dev_otp: Option<String>, // ONLY populated in explicit development mode, NEVER in production
}

pub trait AuthRepository: Send + Sync {
    fn request_otp(&self, email: &str) -> Result<OtpRequestResult>;
    fn verify_otp(&self, email: &str, code: &str) -> Result<EmployeeProfile>;
    fn logout(&self) -> Result<()>;
    fn get_current_user(&self) -> Option<EmployeeProfile>;
    fn get_pending_email(&self) -> Option<String>;
    fn is_development_mode(&self) -> bool;
    fn auth_mode(&self) -> AuthMode;
}

pub fn create_auth_repository(
    mode: AuthMode,
    supabase_url: Option<String>,
    publishable_key: Option<String>,
) -> Result<Box<dyn AuthRepository>> {
    match mode {
        AuthMode::Development => Ok(Box::new(dev_adapter::DevelopmentAuthAdapter::new())),
        AuthMode::SupabaseProduction => {
            let url = supabase_url.ok_or_else(|| {
                AppError::AuthError("Production auth requires VITE_SUPABASE_URL configuration.".to_string())
            })?;
            let key = publishable_key.ok_or_else(|| {
                AppError::AuthError("Production auth requires VITE_SUPABASE_PUBLISHABLE_KEY configuration.".to_string())
            })?;
            Ok(Box::new(supabase_adapter::SupabaseAuthAdapter::new(url, key)))
        }
    }
}
