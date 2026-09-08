use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivityIdentity {
    pub identifier: String,
    pub display_name: String,
    pub activity_type: String,
    pub category: String,
    pub source: String, // "desktop_application" | "browser"
    pub executable: Option<String>,
    pub domain: Option<String>,
}

pub struct ActivityResolver {
    app_mappings: HashMap<String, (String, String)>, // exec_name (lowercase) -> (display_name, category)
    domain_mappings: HashMap<String, (String, String)>, // domain -> (display_name, category)
}

impl ActivityResolver {
    pub fn new() -> Self {
        let mut resolver = Self {
            app_mappings: HashMap::new(),
            domain_mappings: HashMap::new(),
        };
        resolver.load_default_mappings();
        resolver
    }

    fn load_default_mappings(&mut self) {
        // Desktop Applications
        self.register_app("winword.exe", "Microsoft Word", "productivity");
        self.register_app("excel.exe", "Microsoft Excel", "productivity");
        self.register_app("powerpnt.exe", "Microsoft PowerPoint", "productivity");
        self.register_app("code.exe", "Visual Studio Code", "development");
        self.register_app("slack.exe", "Slack", "communication");
        self.register_app("whatsapp.exe", "WhatsApp", "communication");
        self.register_app("teams.exe", "Microsoft Teams", "communication");
        self.register_app("ms-teams.exe", "Microsoft Teams", "communication");
        self.register_app("discord.exe", "Discord", "communication");
        self.register_app("outlook.exe", "Microsoft Outlook", "email");
        self.register_app("notepad.exe", "Notepad", "utilities");
        self.register_app("chrome.exe", "Google Chrome", "browser");
        self.register_app("msedge.exe", "Microsoft Edge", "browser");

        // Browser Websites / Domains
        self.register_domain("mail.google.com", "Gmail", "email");
        self.register_domain("meet.google.com", "Google Meet", "communication");
        self.register_domain("linkedin.com", "LinkedIn", "social");
        self.register_domain("web.whatsapp.com", "WhatsApp Web", "communication");
        self.register_domain("github.com", "GitHub", "development");
        self.register_domain("notion.so", "Notion", "productivity");
        self.register_domain("figma.com", "Figma", "design");
    }

    pub fn register_app(&mut self, exec_name: &str, display_name: &str, category: &str) {
        self.app_mappings.insert(
            exec_name.to_lowercase(),
            (display_name.to_string(), category.to_string()),
        );
    }

    pub fn register_domain(&mut self, domain: &str, display_name: &str, category: &str) {
        self.domain_mappings.insert(
            domain.to_lowercase(),
            (display_name.to_string(), category.to_string()),
        );
    }

    pub fn resolve_application(&self, exec_name: &str) -> ActivityIdentity {
        let clean_exec = exec_name.trim();
        let key = clean_exec.to_lowercase();

        if let Some((disp_name, cat)) = self.app_mappings.get(&key) {
            ActivityIdentity {
                identifier: clean_exec.to_string(),
                display_name: disp_name.clone(),
                activity_type: "desktop_application".to_string(),
                category: cat.clone(),
                source: "desktop_application".to_string(),
                executable: Some(clean_exec.to_string()),
                domain: None,
            }
        } else {
            // Dynamic fallback: strip extension and capitalize
            let name_without_ext = clean_exec
                .trim_end_matches(".exe")
                .trim_end_matches(".EXE");
            let mut chars = name_without_ext.chars();
            let fallback_name = match chars.next() {
                None => clean_exec.to_string(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            };

            ActivityIdentity {
                identifier: clean_exec.to_string(),
                display_name: fallback_name,
                activity_type: "desktop_application".to_string(),
                category: "general_desktop".to_string(),
                source: "desktop_application".to_string(),
                executable: Some(clean_exec.to_string()),
                domain: None,
            }
        }
    }

    pub fn resolve_website(&self, domain_input: &str) -> ActivityIdentity {
        let clean_domain = domain_input.trim().to_lowercase();

        // Exact or subdomain match
        let mut matched_info: Option<(String, String)> = None;

        if let Some((disp, cat)) = self.domain_mappings.get(&clean_domain) {
            matched_info = Some((disp.clone(), cat.clone()));
        } else {
            // Check subdomains (e.g. www.github.com -> github.com)
            for (registered_domain, (disp, cat)) in &self.domain_mappings {
                if clean_domain.ends_with(registered_domain) {
                    matched_info = Some((disp.clone(), cat.clone()));
                    break;
                }
            }
        }

        if let Some((disp_name, cat)) = matched_info {
            ActivityIdentity {
                identifier: clean_domain.clone(),
                display_name: disp_name,
                activity_type: "browser".to_string(),
                category: cat,
                source: "browser".to_string(),
                executable: None,
                domain: Some(clean_domain),
            }
        } else {
            // Dynamic fallback: extract root domain capitalized
            let domain_parts: Vec<&str> = clean_domain.split('.').collect();
            let root_part = if domain_parts.len() >= 2 {
                domain_parts[domain_parts.len() - 2]
            } else {
                &clean_domain
            };
            let mut chars = root_part.chars();
            let fallback_disp = match chars.next() {
                None => clean_domain.clone(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            };

            ActivityIdentity {
                identifier: clean_domain.clone(),
                display_name: fallback_disp,
                activity_type: "browser".to_string(),
                category: "general_web".to_string(),
                source: "browser".to_string(),
                executable: None,
                domain: Some(clean_domain),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_application_resolution() {
        let resolver = ActivityResolver::new();

        let word = resolver.resolve_application("WINWORD.EXE");
        assert_eq!(word.display_name, "Microsoft Word");
        assert_eq!(word.category, "productivity");
        assert_eq!(word.source, "desktop_application");

        let vscode = resolver.resolve_application("Code.exe");
        assert_eq!(vscode.display_name, "Visual Studio Code");
        assert_eq!(vscode.category, "development");

        let custom = resolver.resolve_application("mycustomapp.exe");
        assert_eq!(custom.display_name, "Mycustomapp");
        assert_eq!(custom.category, "general_desktop");
    }

    #[test]
    fn test_website_resolution() {
        let resolver = ActivityResolver::new();

        let gmail = resolver.resolve_website("mail.google.com");
        assert_eq!(gmail.display_name, "Gmail");
        assert_eq!(gmail.category, "email");
        assert_eq!(gmail.source, "browser");

        let github = resolver.resolve_website("www.github.com");
        assert_eq!(github.display_name, "GitHub");
        assert_eq!(github.category, "development");

        let custom_web = resolver.resolve_website("dashboard.internal-tool.org");
        assert_eq!(custom_web.source, "browser");
    }
}
