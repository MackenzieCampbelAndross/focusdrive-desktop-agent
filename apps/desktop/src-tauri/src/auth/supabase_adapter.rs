use crate::auth::{AuthRepository, EmployeeProfile};
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
            "Supabase authentication is not yet connected in this environment. Use Development Auth Mode.".to_string()
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
}
