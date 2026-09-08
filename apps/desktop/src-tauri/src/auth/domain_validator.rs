use crate::errors::{AppError, Result};

pub const ALLOWED_DOMAIN: &str = "purpledrive.com";
pub const ACCESS_DENIED_MESSAGE: &str = "Only PurpleDrive company email addresses are authorized to use the Desktop Agent.";

/// Validates and normalizes an employee work email for Desktop Agent access.
/// - Trims leading/trailing whitespace
/// - Ensures single '@' separator
/// - Ensures non-empty local-part without internal spaces
/// - Lowercases domain
/// - Explicit rule: Domain MUST be exactly "purpledrive.com"
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

    if local_part.contains(' ') || local_part.contains('\t') {
        return Err(AppError::AuthError("Email address contains invalid whitespace characters.".to_string()));
    }

    // Exact domain check specifically for Desktop Agent access
    if domain != ALLOWED_DOMAIN {
        return Err(AppError::AuthError(ACCESS_DENIED_MESSAGE.to_string()));
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
    fn test_rejected_domains_before_otp() {
        // Specifically testing user requirements:
        // person@gmail.com -> denied before OTP
        let gmail_err = validate_and_normalize_email("person@gmail.com").unwrap_err();
        assert_eq!(gmail_err.to_string(), format!("Authentication error: {}", ACCESS_DENIED_MESSAGE));

        // person@quantumpulse.ltd -> denied before OTP
        let qp_err = validate_and_normalize_email("person@quantumpulse.ltd").unwrap_err();
        assert_eq!(qp_err.to_string(), format!("Authentication error: {}", ACCESS_DENIED_MESSAGE));

        // person@purpledrive.co -> denied
        let co_err = validate_and_normalize_email("person@purpledrive.co").unwrap_err();
        assert_eq!(co_err.to_string(), format!("Authentication error: {}", ACCESS_DENIED_MESSAGE));

        // person@sub.purpledrive.com -> denied
        let sub_err = validate_and_normalize_email("person@sub.purpledrive.com").unwrap_err();
        assert_eq!(sub_err.to_string(), format!("Authentication error: {}", ACCESS_DENIED_MESSAGE));

        // person@purpledrive.com.evil.com -> denied
        let evil_err = validate_and_normalize_email("person@purpledrive.com.evil.com").unwrap_err();
        assert_eq!(evil_err.to_string(), format!("Authentication error: {}", ACCESS_DENIED_MESSAGE));
    }
}
