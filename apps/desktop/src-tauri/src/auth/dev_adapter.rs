use std::sync::Mutex;
use std::collections::HashMap;
use crate::auth::{AuthMode, AuthRepository, EmployeeProfile, OtpRequestResult};
use crate::auth::domain_validator::validate_and_normalize_email;
use crate::errors::{AppError, Result};

pub struct DevelopmentAuthAdapter {
    current_user: Mutex<Option<EmployeeProfile>>,
    pending_email: Mutex<Option<String>>,
    active_otps: Mutex<HashMap<String, String>>,
}

impl DevelopmentAuthAdapter {
    pub fn new() -> Self {
        Self {
            current_user: Mutex::new(None),
            pending_email: Mutex::new(None),
            active_otps: Mutex::new(HashMap::new()),
        }
    }
}

impl AuthRepository for DevelopmentAuthAdapter {
    fn request_otp(&self, email: &str) -> Result<OtpRequestResult> {
        // Enforce exact @purpledrive.com domain validation even in development mode!
        let valid_email = validate_and_normalize_email(email)?;

        // Deterministic development OTP based on email local part or standard test code
        let dev_otp = "123456".to_string();

        {
            let mut otps = self.active_otps.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
            otps.insert(valid_email.clone(), dev_otp.clone());
        }
        {
            let mut pending = self.pending_email.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
            *pending = Some(valid_email.clone());
        }

        Ok(OtpRequestResult {
            email: valid_email,
            message: "Development verification code generated.".to_string(),
            dev_otp: Some(dev_otp),
        })
    }

    fn verify_otp(&self, email: &str, code: &str) -> Result<EmployeeProfile> {
        let valid_email = validate_and_normalize_email(email)?;
        let code = code.trim();

        if code.is_empty() {
            return Err(AppError::AuthError("Verification code cannot be empty.".to_string()));
        }

        let is_valid = {
            let otps = self.active_otps.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
            match otps.get(&valid_email) {
                Some(stored) => stored == code,
                None => code == "123456", // standard fallback in dev mode
            }
        };

        if !is_valid {
            return Err(AppError::AuthError("Invalid verification code. Please check your code and try again.".to_string()));
        }

        // Clean up OTP
        {
            let mut otps = self.active_otps.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
            otps.remove(&valid_email);
        }
        {
            let mut pending = self.pending_email.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
            *pending = None;
        }

        let name = valid_email.split('@').next().unwrap_or("Employee");
        let formatted_name = name
            .split('.')
            .map(|s| {
                let mut c = s.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        let profile = EmployeeProfile {
            id: format!("emp_pd_{}", valid_email.replace(['@', '.'], "_")),
            name: if formatted_name.is_empty() { "PurpleDrive Employee".to_string() } else { formatted_name },
            email: valid_email,
            role: "Software Engineer".to_string(),
            department: Some("Engineering".to_string()),
        };

        {
            let mut lock = self.current_user.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
            *lock = Some(profile.clone());
        }

        Ok(profile)
    }

    fn logout(&self) -> Result<()> {
        {
            let mut lock = self.current_user.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
            *lock = None;
        }
        {
            let mut pending = self.pending_email.lock().map_err(|e| AppError::InternalError(e.to_string()))?;
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
    fn test_dev_auth_domain_enforcement() {
        let adapter = DevelopmentAuthAdapter::new();
        // Dev mode CANNOT bypass the purpledrive.com restriction
        assert!(adapter.request_otp("user@gmail.com").is_err());
        assert!(adapter.request_otp("user@purpledrive.co").is_err());

        // Allowed domain passes
        let req = adapter.request_otp("employee@purpledrive.com").unwrap();
        assert_eq!(req.dev_otp.unwrap(), "123456");
    }

    #[test]
    fn test_dev_otp_verification() {
        let adapter = DevelopmentAuthAdapter::new();
        adapter.request_otp("jane.doe@purpledrive.com").unwrap();

        // Invalid code fails
        assert!(adapter.verify_otp("jane.doe@purpledrive.com", "000000").is_err());

        // Valid code succeeds
        let profile = adapter.verify_otp("jane.doe@purpledrive.com", "123456").unwrap();
        assert_eq!(profile.email, "jane.doe@purpledrive.com");
        assert_eq!(adapter.get_current_user().unwrap().email, "jane.doe@purpledrive.com");

        // Logout
        adapter.logout().unwrap();
        assert!(adapter.get_current_user().is_none());
    }
}
