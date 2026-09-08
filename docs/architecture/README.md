# FocusDrive Desktop Agent Architecture

## Overview
The FocusDrive Desktop Agent is a local-first Windows telemetry agent built with Tauri 2, React, TypeScript, and Rust. It collects application and web usage telemetry only during active employee shifts.

```
                         FOCUSDRIVE
                              │
                         SUPABASE
                              │
               ┌──────────────┼──────────────┐
               │              │              │
             Auth         PostgreSQL    Edge Functions
               │              │              │
               └──────────────┼──────────────┘
                              │
                           HTTPS
                              │
                              ▼
                    DESKTOP AGENT
                         Tauri 2
                              │
                    React + TypeScript
                              │
                         Rust Core
                              │
       ┌──────────────────────┼──────────────────────┐
       │                      │                      │
 Windows Telemetry      Activity Engine          Sync Engine
       │                      │                      │
       │                   SQLite                   │
       │                                             │
       └──────────────────────┬──────────────────────┘
                              │
                       Native Messaging
                              │
                 ┌────────────┴────────────┐
                 │                         │
          Chrome Extension          Edge Extension
                 │                         │
                 └────────────┬────────────┘
                              │
                         Active Domain
```

## Guiding Architectural Principles
1. **Explicit Shift Control**: Telemetry is strictly active only when employee explicitly clicks **START SHIFT**.
2. **Local-First Resilience**: Telemetry is saved to local SQLite first. Sync Engine asynchronously batch-pushes to Supabase.
3. **No Invasive Scraping**: Privacy boundaries strictly limit telemetry to application executable identity and active browser domain metadata.
