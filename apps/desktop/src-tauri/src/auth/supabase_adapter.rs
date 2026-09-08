use std::sync::Mutex;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use crate::auth::{AuthMode, AuthRepository, EmployeeProfile};
use crate::errors::{AppError, Result};

#[derive(Debug, Clone, Deserialize)]
struct SupabaseAuthResponse {
    access_token: String,
    user: SupabaseUser,
}

#[derive(Debug, Clone, Deserialize)]
struct SupabaseUser {
    id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct SupabaseEmployeeRow {
    id: String,
    name: String,
    email: String,
    role: String,
    department: Option<String>,
}

pub struct SupabaseAuthAdapter {
    supabase_url: String,
    publishable_key: String,
    client: Client,
    current_user: Mutex<Option<EmployeeProfile>>,
    access_token: Mutex<Option<String>>,
}

impl SupabaseAuthAdapter {
    pub fn new(supabase_url: String, publishable_key: String) -> Self {
        Self {
            supabase_url: supabase_url.trim_end_matches('/').to_string(),
            publishable_key,
            client: Client::new(),
            current_user: Mutex::new(None),
            access_token: Mutex::new(None),
        }
    }

    pub fn get_access_token(&self) -> Option<String> {
        self.access_token.lock().ok()?.clone()
    }

    pub fn get_supabase_url(&self) -> &str {
        &self.supabase_url
    }

    pub fn get_publishable_key(&self) -> &str {
        &self.publishable_key
    }
}

impl AuthRepository for SupabaseAuthAdapter {
    fn login(&self, email: &str, password: &str) -> Result<EmployeeProfile> {
        let email = email.trim();
        if email.is_empty() {
            return Err(AppError::AuthError("Email cannot be empty".to_string()));
        }
        if password.is_empty() {
            return Err(AppError::AuthError("Password cannot be empty".to_string()));
        }

        let auth_endpoint = format!("{}/auth/v1/token?grant_type=password", self.supabase_url);

        let body = json!({
            "email": email,
            "password": password,
        });

        let client = self.client.clone();
        let pub_key = self.publishable_key.clone();
        let sub_url = self.supabase_url.clone();
        let email_owned = email.to_string();

        let (token, auth_uid) = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let res = client
                    .post(&auth_endpoint)
                    .header("apikey", &pub_key)
                    .header("Content-Type", "application/json")
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| AppError::NetworkError(format!("Failed to connect to Supabase auth: {}", e)))?;

                let status = res.status();
                let res_text: String = res.text().await
                    .map_err(|e| AppError::NetworkError(format!("Failed to read auth response: {}", e)))?;

                if !status.is_success() {
                    return Err(AppError::AuthError(format!(
                        "Supabase authentication failed (HTTP {}): {}",
                        status.as_u16(),
                        res_text
                    )));
                }

                let auth_data: SupabaseAuthResponse = serde_json::from_str(&res_text)
                    .map_err(|e| AppError::AuthError(format!("Invalid auth token response: {}", e)))?;

                Ok::<_, AppError>((auth_data.access_token, auth_data.user.id))
            })
        })?;

        // Query public.employees matching auth_user_id or email
        let emp_endpoint = format!(
            "{}/rest/v1/employees?select=*&or=(auth_user_id.eq.{},email.eq.{})",
            sub_url, auth_uid, email_owned
        );

        let client2 = self.client.clone();
        let pub_key2 = self.publishable_key.clone();
        let token_clone = token.clone();

        let employees: Vec<SupabaseEmployeeRow> = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let emp_res = client2
                    .get(&emp_endpoint)
                    .header("apikey", &pub_key2)
                    .header("Authorization", format!("Bearer {}", token_clone))
                    .send()
                    .await
                    .map_err(|e| AppError::NetworkError(format!("Failed to fetch employee record: {}", e)))?;

                let emp_status = emp_res.status();
                let emp_text: String = emp_res.text().await
                    .map_err(|e| AppError::NetworkError(format!("Failed to read employee response: {}", e)))?;

                if !emp_status.is_success() {
                    return Err(AppError::AuthError(format!(
                        "Failed to query employee profile (HTTP {}): {}",
                        emp_status.as_u16(),
                        emp_text
                    )));
                }

                serde_json::from_str(&emp_text)
                    .map_err(|e| AppError::AuthError(format!("Invalid employee data structure: {}", e)))
            })
        })?;

        let profile = if let Some(first) = employees.into_iter().next() {
            EmployeeProfile {
                id: first.id,
                name: first.name,
                email: first.email,
                role: first.role,
                department: first.department,
            }
        } else {
            EmployeeProfile {
                id: auth_uid,
                name: email.split('@').next().unwrap_or("Employee").to_string(),
                email: email.to_string(),
                role: "employee".to_string(),
                department: None,
            }
        };

        if let Ok(mut u_lock) = self.current_user.lock() {
            *u_lock = Some(profile.clone());
        }
        if let Ok(mut t_lock) = self.access_token.lock() {
            *t_lock = Some(token);
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
        Ok(())
    }

    fn get_current_user(&self) -> Option<EmployeeProfile> {
        self.current_user.lock().ok()?.clone()
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
    }
}
