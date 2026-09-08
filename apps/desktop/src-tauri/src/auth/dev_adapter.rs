use std::sync::Mutex;
use crate::auth::{AuthMode, AuthRepository, EmployeeProfile};
use crate::errors::{AppError, Result};

pub struct DevelopmentAuthAdapter {
    current_user: Mutex<Option<EmployeeProfile>>,
}

impl DevelopmentAuthAdapter {
    pub fn new() -> Self {
        Self {
            current_user: Mutex::new(None),
        }
    }

    pub fn with_initial_user(user: Option<EmployeeProfile>) -> Self {
        Self {
            current_user: Mutex::new(user),
        }
    }
}

impl AuthRepository for DevelopmentAuthAdapter {
    fn login(&self, email: &str, _password: &str) -> Result<EmployeeProfile> {
        if email.trim().is_empty() {
            return Err(AppError::AuthError("Email cannot be empty".to_string()));
        }

        // Development mode test profiles
        let profile = match email.to_lowercase().as_str() {
            "sarah@focusdrive.dev" => EmployeeProfile {
                id: "emp_dev_002".to_string(),
                name: "Sarah Miller".to_string(),
                email: "sarah@focusdrive.dev".to_string(),
                role: "Product Manager".to_string(),
                department: Some("Product".to_string()),
            },
            _ => EmployeeProfile {
                id: "emp_dev_001".to_string(),
                name: if email.contains('@') {
                    email.split('@').next().unwrap_or("Dev Employee").to_string()
                } else {
                    "Alex Engineer".to_string()
                },
                email: email.to_string(),
                role: "Senior Engineer [DEV]".to_string(),
                department: Some("Engineering".to_string()),
            },
        };

        let mut lock = self.current_user.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
        *lock = Some(profile.clone());
        Ok(profile)
    }

    fn logout(&self) -> Result<()> {
        let mut lock = self.current_user.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
        *lock = None;
        Ok(())
    }

    fn get_current_user(&self) -> Option<EmployeeProfile> {
        self.current_user.lock().ok()?.clone()
    }

    fn is_development_mode(&self) -> bool {
        true
    }

    fn auth_mode(&self) -> AuthMode {
        AuthMode::Development
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dev_auth_gating() {
        let adapter = DevelopmentAuthAdapter::new();
        assert_eq!(adapter.auth_mode(), AuthMode::Development);
        assert!(adapter.is_development_mode());

        let user = adapter.login("alex@focusdrive.dev", "pwd").unwrap();
        assert_eq!(user.email, "alex@focusdrive.dev");
        assert_eq!(adapter.get_current_user().unwrap().email, "alex@focusdrive.dev");

        adapter.logout().unwrap();
        assert!(adapter.get_current_user().is_none());
    }
}
