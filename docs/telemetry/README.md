# Telemetry Specification

## Active Telemetry Rules
1. Telemetry collects data ONLY when state is `WORKING`.
2. Telemetry pauses completely when state is `BREAK` or `SHIFT_SUMMARY` or `SHIFT_READY`.
3. Idle detection triggers automatic break after 60 seconds of zero keyboard/mouse input.
4. Activity intervals are normalized and aggregated into total integer seconds per activity category per shift.
