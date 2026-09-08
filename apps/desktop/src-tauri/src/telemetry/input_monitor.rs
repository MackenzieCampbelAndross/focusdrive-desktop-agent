#[cfg(target_os = "windows")]
pub fn get_last_input_idle_milliseconds() -> u64 {
    use windows_sys::Win32::System::SystemInformation::GetTickCount64;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

    unsafe {
        let mut lii = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };

        if GetLastInputInfo(&mut lii) != 0 {
            let tick_count = GetTickCount64();
            let last_input_time = lii.dwTime as u64;
            // Handle GetTickCount overflow / rollover
            if tick_count >= last_input_time {
                tick_count - last_input_time
            } else {
                0
            }
        } else {
            0
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_last_input_idle_milliseconds() -> u64 {
    0
}

/// Product Rule: Idle threshold is 60 seconds (60,000 ms).
/// Inactivity beyond 60s counts as automatic break time.
/// The first 60 seconds are NOT counted as break time.
pub const DEFAULT_IDLE_THRESHOLD_SECONDS: u64 = 60;

pub fn calculate_automatic_break_duration_seconds(total_idle_seconds: u64, threshold_seconds: u64) -> u64 {
    if total_idle_seconds > threshold_seconds {
        total_idle_seconds - threshold_seconds
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idle_threshold_calculations() {
        let threshold = 60;

        // 59 seconds idle -> 0 break seconds
        assert_eq!(calculate_automatic_break_duration_seconds(59, threshold), 0);

        // 60 seconds idle -> 0 break seconds (exact threshold)
        assert_eq!(calculate_automatic_break_duration_seconds(60, threshold), 0);

        // 61 seconds idle -> 1 second break
        assert_eq!(calculate_automatic_break_duration_seconds(61, threshold), 1);

        // 5 minutes idle (300s) -> 240 seconds break (4 minutes)
        assert_eq!(calculate_automatic_break_duration_seconds(300, threshold), 240);

        // 7 minutes 30 seconds idle (450s) -> 390 seconds break (6 minutes 30 seconds)
        assert_eq!(calculate_automatic_break_duration_seconds(450, threshold), 390);

        // Long idle: 1 hour (3600s) -> 3540 seconds break (59 minutes)
        assert_eq!(calculate_automatic_break_duration_seconds(3600, threshold), 3540);
    }
}
