pub mod dev_adapter;
pub mod supabase_adapter;

use serde::{Deserialize, Serialize};
use crate::errors::Result;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmployeeProfile {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub department: Option<String>,
}

pub trait AuthRepository: Send + Sync {
    fn login(&self, email: &str, password: &str) -> Result<EmployeeProfile>;
    fn logout(&self) -> Result<()>;
    fn get_current_user(&self) -> Option<EmployeeProfile>;
    fn is_development_mode(&self) -> bool;
}
