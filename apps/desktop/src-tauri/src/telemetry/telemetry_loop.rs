// src/telemetry/telemetry_loop.rs
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use crate::session::SessionManager;
use crate::config::telemetry::TelemetryConfig;
use crate::telemetry::{foreground_window, input_monitor, activity_aggregator::ActivityAggregator};
use crate::state::SessionState;
use tracing::info;

/// Runs the telemetry loop in a background Tokio task.
/// The loop polls every interval_ms (default 1000 ms).
/// It updates activity aggregation, handles idle detection and automatic-break transitions.
pub fn spawn_telemetry_loop(
    session_manager: Arc<Mutex<SessionManager>>,
    activity_aggregator: Arc<Mutex<ActivityAggregator>>,
    config: TelemetryConfig,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let interval = Duration::from_millis(config.interval_ms);
        loop {
            // Determine current state - exit loop if no telemetry is required.
            let current_state = {
                let sm = session_manager.lock().await;
                sm.current_state()
            };
            if !matches!(current_state, SessionState::Working | SessionState::AutomaticBreak | SessionState::ManualBreak) {
                break;
            }

            // 1. Foreground application.
            let proc_info = foreground_window::get_foreground_process_info();
            // 2. Idle time (ms).
            let idle_ms = input_monitor::get_last_input_idle_milliseconds();
            let idle_seconds = idle_ms / 1000;

            // 3. IDs for DB writes.
            let (work_id, employee_id) = {
                let sm = session_manager.lock().await;
                if let Some(sess) = sm.active_session() {
                    (sess.id.clone(), sess.employee_id.clone())
                } else {
                    (String::new(), String::new())
                }
            };

            // 4. Update aggregation (desktop app only - browser data arrives via native host).
            if !work_id.is_empty() {
                let mut agg = activity_aggregator.lock().await;
                if let Some(info) = proc_info {
                    let _ = agg.update_foreground_application(&work_id, &employee_id, &info.executable_name);
                }
            }

            // 5. Automatic-break logic.
            {
                let mut sm = session_manager.lock().await;
                match sm.current_state() {
                    SessionState::Working => {
                        if idle_seconds >= 60 {
                            let _ = sm.start_automatic_break();
                            drop(sm);
                            let mut agg = activity_aggregator.lock().await;
                            let _ = agg.pause_telemetry(&work_id, &employee_id);
                        }
                    }
                    SessionState::AutomaticBreak => {
                        if idle_seconds < 60 {
                            let _ = sm.end_automatic_break();
                            drop(sm);
                            let mut agg = activity_aggregator.lock().await;
                            let _ = agg.resume_telemetry(&work_id, &employee_id);
                        }
                    }
                    _ => {}
                }
            }

            // 6. Wait for next tick.
            tokio::time::sleep(interval).await;
        }
        info!("Telemetry loop terminated");
    })
}
