// src/telemetry/application_resolver.rs
use std::collections::HashMap;
use lazy_static::lazy_static;
use crate::telemetry::activity_resolver::ActivityIdentity;

/// Central registry mapping executable names (lowercase) to friendly application names.
pub struct ApplicationResolver {
    map: HashMap<String, &'static str>,
}

impl ApplicationResolver {
    pub fn new() -> Self {
        lazy_static! {
            static ref MAP: HashMap<String, &'static str> = {
                let mut m = HashMap::new();
                m.insert("winword.exe".to_string(), "Microsoft Word");
                m.insert("excel.exe".to_string(), "Microsoft Excel");
                m.insert("powerpnt.exe".to_string(), "Microsoft PowerPoint");
                m.insert("outlook.exe".to_string(), "Microsoft Outlook");
                m.insert("code.exe".to_string(), "Visual Studio Code");
                m.insert("code.exe".to_string(), "Visual Studio Code");
                m.insert("teams.exe".to_string(), "Microsoft Teams");
                m.insert("slack.exe".to_string(), "Slack");
                m.insert("whatsapp.exe".to_string(), "WhatsApp");
                // Add more known mappings as needed.
                m
            };
        }
        Self { map: MAP.clone() }
    }

    /// Resolve an executable name to an `ActivityIdentity`.
    pub fn resolve(&self, exec_name: &str) -> ActivityIdentity {
        let key = exec_name.to_ascii_lowercase();
        let display = self.map.get(&key).unwrap_or(&exec_name);
        ActivityIdentity {
            activity_type: "desktop_application".to_string(),
            identifier: key.clone(),
            display_name: display.to_string(),
        }
    }
}
