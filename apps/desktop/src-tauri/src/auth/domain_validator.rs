use crate::errors::{AppError, Result};

pub const ALLOWED_DOMAIN: &str = "purpledrive.com";

/// Validates and normalizes an employee work email.
/// - Trims whitespace
/// - Ensures single '@' separator
/// - Ensures non-empty local part
/// - Normalizes domain to lowercase
/// - Hard restriction: domain MUST be exactly "purpledrive.com"
pub fn validate_and_normalize_email(email_input: &str) -> Result<String> {
    let trimmed = email_input.trim();
    if trimmed.is_empty() {
        return Err(AppError::AuthError("Email address cannot be empty.".to_string()));
    }

    let parts: Vec<&str> = trimmed.split('@').collect();
    if parts.len() != 2 {
        return Err(AppError::AuthError("Invalid email address format. Expected local-part@domain.".to_string()));
    }

    let local_part = parts[0].trim();
    let domain = parts[1].trim().to_lowercase();

    if local_part.is_empty() {
        return Err(AppError::AuthError("Email username/local-part cannot be empty.".to_string()));
    }

    // Exact domain check
    if domain != ALLOWED_DOMAIN {
        return Err(AppError::AuthError(format!(
            "Access restricted: Only @{} work email addresses are permitted. '{}' is not authorized.",
            ALLOWED_DOMAIN, domain
        )));
    }

    // Syntax check for local part
    if local_part.contains(' ') || local_part.contains('\t') {
        return Err(AppError::AuthError("Email address contains invalid whitespace characters.".to_string()));
    }

    Ok(format!("{}@{}", local_part, domain))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_purpledrive_emails() {
        assert_eq!(
            validate_and_normalize_email("employee@purpledrive.com").unwrap(),
            "employee@purpledrive.com"
        );
        assert_eq!(
            validate_and_normalize_email("EMPLOYEE@PURPLEDRIVE.COM").unwrap(),
            "EMPLOYEE@purpledrive.com"
        );
        assert_eq!(
            validate_and_normalize_email("  john.doe@purpledrive.com  ").unwrap(),
            "john.doe@purpledrive.com"
        );
    }

    #[test]
    fn test_rejected_domains() {
        assert!(validate_and_normalize_email("employee@gmail.com").is_err());
        assert!(validate_and_normalize_email("employee@google.com").is_err());
        assert!(validate_and_normalize_email("employee@purpledrive.co").is_err());
        assert!(validate_and_normalize_email("employee@purpledrive.com.evil.com").is_err());
        assert!(validate_and_normalize_email("employee@sub.purpledrive.com").is_err());
        assert!(validate_and_normalize_email("employee@purpledrive.com.example.com").is_err());
        assert!(validate_and_normalize_email("not-an-email").is_err());
        assert!(validate_and_normalize_email("@purpledrive.com").is_err());
        assert!(validate_and_normalize_email("   ").is_err());
    }
}
