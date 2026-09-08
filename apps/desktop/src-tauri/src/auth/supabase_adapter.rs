use crate::auth::{AuthMode, AuthRepository, EmployeeProfile};
use crate::errors::{AppError, Result};

pub struct SupabaseAuthAdapter {
    _supabase_url: String,
    _anon_key: String,
}

impl SupabaseAuthAdapter {
    pub fn new(supabase_url: String, anon_key: String) -> Self {
        Self {
            _supabase_url: supabase_url,
            _anon_key: anon_key,
        }
    }
}

impl AuthRepository for SupabaseAuthAdapter {
    fn login(&self, _email: &str, _password: &str) -> Result<EmployeeProfile> {
        Err(AppError::AuthError(
            format!(
                "Supabase production authentication target ({}) is not yet connected. Configure credentials in SUPABASE_SETUP.md.",
                self._supabase_url
            )
        ))
    }

    fn logout(&self) -> Result<()> {
        Ok(())
    }

    fn get_current_user(&self) -> Option<EmployeeProfile> {
        None
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
            "anon-key".to_string(),
        );
        assert_eq!(adapter.auth_mode(), AuthMode::SupabaseProduction);
        assert!(!adapter.is_development_mode());
        assert!(adapter.login("test@focusdrive.dev", "pwd").is_err());
    }
}
