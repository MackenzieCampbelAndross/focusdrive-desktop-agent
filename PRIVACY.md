# Privacy Specification

## Telemetry Boundaries & Data Collection Scope

FocusDrive Desktop Agent is designed as a privacy-respecting employee telemetry client.

### Explicit Shift Activation
Telemetry collection occurs **ONLY** during an active shift (when state is `WORKING`). No background monitoring occurs in `LOGGED_OUT`, `SHIFT_READY`, `BREAK`, or `SHIFT_SUMMARY` states.

### Data Collected
- Active foreground executable name (e.g. `WINWORD.EXE`)
- Active browser domain name (e.g. `mail.google.com`)
- Duration in integer seconds
- Device identity metadata (OS, computer name, installation UUID)
- Work session and break session timestamps

### Data EXCLUDED (Never Collected)
- Screenshots or screen recordings
- Keystrokes or input logging
- Clipboard contents
- Passwords or credentials
- Private messages, email bodies, or chat logs
- Full URL query parameters or page paths
- DOM element contents or form inputs
- Audio or video feeds
