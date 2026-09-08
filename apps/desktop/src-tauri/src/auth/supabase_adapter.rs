use std::sync::Mutex;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use crate::auth::{AuthMode, AuthRepository, EmployeeProfile, OtpRequestResult};
use crate::auth::domain_validator::validate_and_normalize_email;
use crate::errors::{AppError, Result};

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct SupabaseVerifyOtpResponse {
    access_token: String,
    user: SupabaseUser,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct SupabaseUser {
    id: String,
    email: Option<String>,
}

/// Matches the real production public.employees schema:
/// id TEXT, name TEXT, email TEXT, initials TEXT, role TEXT, title TEXT,
/// team_id TEXT, manager_id TEXT, timezone TEXT, shift_start TEXT,
/// shift_end TEXT, shift_overnight BOOLEAN, shift_days TEXT, status TEXT, hired_on DATE
#[derive(Debug, Clone, Deserialize)]
pub struct ProductionEmployeeRecord {
    pub id: String,
    pub name: String,
    pub email: String,
    pub initials: Option<String>,
    pub role: Option<String>,
    pub title: Option<String>,
    pub team_id: Option<String>,
    pub manager_id: Option<String>,
    pub timezone: Option<String>,
    pub status: Option<String>,
}

pub struct SupabaseAuthAdapter {
    supabase_url: String,
    publishable_key: String,
    client: Client,
    current_user: Mutex<Option<EmployeeProfile>>,
    pending_email: Mutex<Option<String>>,
    access_token: Mutex<Option<String>>,
}

impl SupabaseAuthAdapter {
    pub fn new(supabase_url: String, publishable_key: String) -> Self {
        Self {
            supabase_url: supabase_url.trim_end_matches('/').to_string(),
            publishable_key,
            client: Client::new(),
            current_user: Mutex::new(None),
            pending_email: Mutex::new(None),
            access_token: Mutex::new(None),
        }
    }

    pub fn get_access_token(&self) -> Option<String> {
        self.access_token.lock().ok()?.clone()
    }
}

impl AuthRepository for SupabaseAuthAdapter {
    fn request_otp(&self, email: &str) -> Result<OtpRequestResult> {
        // 1. Rust-level domain restriction: strictly @purpledrive.com check
        let valid_email = validate_and_normalize_email(email)?;

        let otp_endpoint = format!("{}/auth/v1/otp", self.supabase_url);
        let body = json!({
            "email": valid_email,
            "create_user": false, // Do NOT automatically create unknown users; existing employee required
        });

        let client = self.client.clone();
        let pub_key = self.publishable_key.clone();
        let target_email = valid_email.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let res = client
                    .post(&otp_endpoint)
                    .header("apikey", &pub_key)
                    .header("Content-Type", "application/json")
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| AppError::NetworkError(format!("Failed to connect to Supabase auth service: {}", e)))?;

                let status = res.status();
                if !status.is_success() {
                    let err_text = res.text().await.unwrap_or_default();
                    return Err(AppError::AuthError(format!(
                        "Failed to send verification code (HTTP {}): {}",
                        status.as_u16(),
                        err_text
                    )));
                }

                Ok::<_, AppError>(())
            })
        })?;

        {
            let mut pending = self.pending_email.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
            *pending = Some(valid_email.clone());
        }

        Ok(OtpRequestResult {
            email: target_email,
            message: "A verification code has been sent to your @purpledrive.com email address.".to_string(),
            dev_otp: None, // Production NEVER exposes OTPs
        })
    }

    fn verify_otp(&self, email: &str, token: &str) -> Result<EmployeeProfile> {
        let valid_email = validate_and_normalize_email(email)?;
        let token = token.trim();
        if token.is_empty() {
            return Err(AppError::AuthError("Verification code cannot be empty.".to_string()));
        }

        let verify_endpoint = format!("{}/auth/v1/verify", self.supabase_url);
        let body = json!({
            "type": "email",
            "email": valid_email,
            "token": token,
        });

        let client = self.client.clone();
        let pub_key = self.publishable_key.clone();
        let sub_url = self.supabase_url.clone();
        let target_email = valid_email.clone();

        let access_token = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let res = client
                    .post(&verify_endpoint)
                    .header("apikey", &pub_key)
                    .header("Content-Type", "application/json")
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| AppError::NetworkError(format!("Failed to connect to Supabase auth service: {}", e)))?;

                let status = res.status();
                let res_text = res.text().await
                    .map_err(|e| AppError::NetworkError(format!("Failed to read auth response: {}", e)))?;

                if !status.is_success() {
                    return Err(AppError::AuthError(format!(
                        "OTP verification failed (HTTP {}): {}",
                        status.as_u16(),
                        res_text
                    )));
                }

                let auth_data: SupabaseVerifyOtpResponse = serde_json::from_str(&res_text)
                    .map_err(|e| AppError::AuthError(format!("Invalid auth token response: {}", e)))?;

                Ok::<_, AppError>(auth_data.access_token)
            })
        })?;

        // Identity relationship: auth.users.email -> public.employees.email
        // Query public.employees directly by normalized email using the actual production schema
        let emp_endpoint = format!(
            "{}/rest/v1/employees?select=id,name,email,initials,role,title,team_id,manager_id,timezone,status&email=ilike.{}",
            sub_url, target_email
        );

        let client2 = self.client.clone();
        let pub_key2 = self.publishable_key.clone();
        let token_clone = access_token.clone();

        let employees: Vec<ProductionEmployeeRecord> = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let emp_res = client2
                    .get(&emp_endpoint)
                    .header("apikey", &pub_key2)
                    .header("Authorization", format!("Bearer {}", token_clone))
                    .send()
                    .await
                    .map_err(|e| AppError::NetworkError(format!("Failed to query employee profile: {}", e)))?;

                let emp_status = emp_res.status();
                let emp_text = emp_res.text().await
                    .map_err(|e| AppError::NetworkError(format!("Failed to read employee response: {}", e)))?;

                if !emp_status.is_success() {
                    return Err(AppError::AuthError(format!(
                        "Employee lookup error (HTTP {}): {}",
                        emp_status.as_u16(),
                        emp_text
                    )));
                }

                serde_json::from_str(&emp_text)
                    .map_err(|e| AppError::AuthError(format!("Invalid employee data structure: {}", e)))
            })
        })?;

        let employee_rec = match employees.into_iter().next() {
            Some(emp) => emp,
            None => {
                return Err(AppError::AuthError(format!(
                    "No authorized employee account found matching '{}'. Contact your PurpleDrive administrator.",
                    target_email
                )));
            }
        };

        // Determine authorization using existing FocusDrive employee status semantics:
        // Statuses like 'offline', 'online', 'active' are eligible.
        // Explicitly deactivated statuses like 'terminated', 'suspended', 'inactive' are not.
        if let Some(ref st) = employee_rec.status {
            let normalized_status = st.trim().to_lowercase();
            if normalized_status == "terminated" || normalized_status == "suspended" || normalized_status == "inactive" || normalized_status == "deactivated" {
                return Err(AppError::AuthError(format!(
                    "Employee account for '{}' is currently {} and unauthorized for Desktop Agent access.",
                    target_email, st
                )));
            }
        }

        let profile = EmployeeProfile {
            id: employee_rec.id,
            name: employee_rec.name,
            email: employee_rec.email,
            role: employee_rec.role.or(employee_rec.title).unwrap_or_else(|| "Employee".to_string()),
            department: employee_rec.team_id,
        };

        if let Ok(mut u_lock) = self.current_user.lock() {
            *u_lock = Some(profile.clone());
        }
        if let Ok(mut t_lock) = self.access_token.lock() {
            *t_lock = Some(access_token);
        }
        if let Ok(mut pending) = self.pending_email.lock() {
            *pending = None;
        }

        Ok(profile)
    }

    fn logout(&self) -> Result<()> {
        if let Ok(mut u_lock) = self.current_user.lock() {
            *u_lock = None;
        }
        if let Ok(mut t_lock) = self.access_token.lock() {
            *t_lock = None;
        }
        if let Ok(mut pending) = self.pending_email.lock() {
            *pending = None;
        }
        Ok(())
    }

    fn get_current_user(&self) -> Option<EmployeeProfile> {
        self.current_user.lock().ok()?.clone()
    }

    fn get_pending_email(&self) -> Option<String> {
        self.pending_email.lock().ok()?.clone()
    }

    fn is_development_mode(&self) -> bool {
        false
    }

    fn auth_mode(&self) -> AuthMode {
        AuthMode::SupabaseProduction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supabase_auth_mode_isolation() {
        let adapter = SupabaseAuthAdapter::new(
            "https://test.supabase.co".to_string(),
            "pub-key".to_string(),
        );
        assert_eq!(adapter.auth_mode(), AuthMode::SupabaseProduction);
        assert!(!adapter.is_development_mode());
        assert!(adapter.request_otp("user@gmail.com").is_err());
        assert!(adapter.request_otp("user@quantumpulse.ltd").is_err());
    }
}
