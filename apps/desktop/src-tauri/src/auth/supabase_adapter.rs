use std::sync::Mutex;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use crate::auth::{AuthMode, AuthRepository, EmployeeProfile, OtpRequestResult};
use crate::auth::domain_validator::validate_and_normalize_email;
use crate::errors::{AppError, Result};

#[derive(Debug, Clone, Deserialize)]
struct SupabaseVerifyOtpResponse {
    access_token: String,
    user: SupabaseUser,
}

#[derive(Debug, Clone, Deserialize)]
struct SupabaseUser {
    id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct SupabaseEmployeeRow {
    #[serde(alias = "employee_id")]
    id: String,
    #[serde(alias = "employee_name")]
    name: String,
    #[serde(alias = "employee_email")]
    email: String,
    #[serde(alias = "employee_role")]
    role: String,
    #[serde(alias = "employee_department")]
    department: Option<String>,
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
        // 1. Rust-level domain restriction: exact @purpledrive.com check
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

        let (access_token, auth_uid) = tokio::task::block_in_place(|| {
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

                Ok::<_, AppError>((auth_data.access_token, auth_data.user.id))
            })
        })?;

        // 1. Try secure link function first (if migration 20260908184500 is applied)
        let rpc_endpoint = format!("{}/rest/v1/rpc/link_authenticated_desktop_employee", sub_url);
        let client_rpc = self.client.clone();
        let pub_key_rpc = self.publishable_key.clone();
        let token_rpc = access_token.clone();

        let maybe_profile = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let rpc_res = client_rpc
                    .post(&rpc_endpoint)
                    .header("apikey", &pub_key_rpc)
                    .header("Authorization", format!("Bearer {}", token_rpc))
                    .header("Content-Type", "application/json")
                    .json(&json!({}))
                    .send()
                    .await;

                if let Ok(resp) = rpc_res {
                    if resp.status().is_success() {
                        if let Ok(rows) = resp.json::<Vec<SupabaseEmployeeRow>>().await {
                            if let Some(first) = rows.into_iter().next() {
                                return Some(EmployeeProfile {
                                    id: first.id,
                                    name: first.name,
                                    email: first.email,
                                    role: first.role,
                                    department: first.department,
                                });
                            }
                        }
                    }
                }
                None
            })
        });

        // 2. Fallback to direct employees table query (compatible with existing FocusDrive schema)
        let profile = match maybe_profile {
            Some(p) => p,
            None => {
                let emp_endpoint = format!(
                    "{}/rest/v1/employees?select=*&or=(auth_user_id.eq.{},email.eq.{})",
                    sub_url, auth_uid, target_email
                );

                let client2 = self.client.clone();
                let pub_key2 = self.publishable_key.clone();
                let token_clone = access_token.clone();

                let employees: Vec<SupabaseEmployeeRow> = tokio::task::block_in_place(|| {
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

                if let Some(first) = employees.into_iter().next() {
                    EmployeeProfile {
                        id: first.id,
                        name: first.name,
                        email: first.email,
                        role: first.role,
                        department: first.department,
                    }
                } else {
                    return Err(AppError::AuthError(format!(
                        "No authorized employee account found matching '{}'. Contact your PurpleDrive administrator.",
                        target_email
                    )));
                }
            }
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
    }
}
