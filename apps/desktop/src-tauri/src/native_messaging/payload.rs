use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeTabActivePayload {
    pub r#type: String, // "tab_active"
    pub browser: String, // "chrome" | "edge"
    pub domain: String,
    pub timestamp: String,
}

pub fn validate_native_payload(json_str: &str) -> Option<NativeTabActivePayload> {
    if json_str.len() > 65536 {
        // Enforce 64 KB size limit on payload
        return None;
    }

    if let Ok(payload) = serde_json::from_str::<NativeTabActivePayload>(json_str) {
        if payload.r#type == "tab_active" && !payload.domain.trim().is_empty() {
            let clean_domain = payload.domain.trim().to_lowercase();
            // Basic domain format validation (no spaces, contains valid characters)
            if !clean_domain.contains(' ') && clean_domain.contains('.') {
                return Some(NativeTabActivePayload {
                    domain: clean_domain,
                    ..payload
                });
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_native_payload() {
        let valid_json = r#"{
            "type": "tab_active",
            "browser": "chrome",
            "domain": "mail.google.com",
            "timestamp": "2026-09-08T12:00:00Z"
        }"#;

        let parsed = validate_native_payload(valid_json).unwrap();
        assert_eq!(parsed.domain, "mail.google.com");
        assert_eq!(parsed.browser, "chrome");
    }

    #[test]
    fn test_reject_invalid_native_payload() {
        let invalid_type = r#"{"type": "unknown", "browser": "chrome", "domain": "mail.google.com", "timestamp": ""}"#;
        assert!(validate_native_payload(invalid_type).is_none());

        let invalid_domain = r#"{"type": "tab_active", "browser": "chrome", "domain": "invalid domain name", "timestamp": ""}"#;
        assert!(validate_native_payload(invalid_domain).is_none());
    }
}
