// src/config/telemetry.rs
use serde::Deserialize;

/// Configuration for telemetry loop.
#[derive(Debug, Deserialize, Clone)]
pub struct TelemetryConfig {
    /// Polling interval in milliseconds. Default: 1000 ms.
    pub interval_ms: u64,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self { interval_ms: 1000 }
    }
}

impl TelemetryConfig {
    /// Load from environment variable `FOCUSDRIVE_TELEMETRY_INTERVAL_MS`.
    pub fn from_env() -> Self {
        if let Ok(val) = std::env::var("FOCUSDRIVE_TELEMETRY_INTERVAL_MS") {
            if let Ok(ms) = val.parse::<u64>() {
                return Self { interval_ms: ms };
            }
        }
        Self::default()
    }
}
