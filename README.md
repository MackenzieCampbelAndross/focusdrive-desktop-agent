# FocusDrive Desktop Agent

> **Workstation Telemetry Layer for FocusDrive**

FocusDrive Desktop Agent is a secure, local-first Windows desktop telemetry client engineered with **Tauri 2**, **React**, **TypeScript**, **Vite**, **Rust**, and **SQLite**. It serves as the workstation telemetry provider for FocusDrive, tracking employee active application and browser domain durations during explicitly started work shifts.

## Core Features & Workflow

- **Explicit Shift Control**: Telemetry never records activity prior to employee clicking `START SHIFT`.
- **Centrally Controlled State Machine**: `LOGGED_OUT` → `SHIFT_READY` → `WORKING` ⇄ `BREAK` → `SHIFT_SUMMARY`.
- **Automatic Idle Detection**: Detects system inactivity using Windows Win32 APIs and shifts to automatic break after 60 seconds.
- **Application & Website Telemetry**: Resolves active foreground applications and browser tabs (via Chrome/Edge companion extensions and Native Messaging).
- **Local-First Resiliency**: All activity counts persist locally in SQLite first before batch synchronization to Supabase PostgreSQL.

## Repository Architecture

```
focusdrive-desktop-agent/
├── apps/
│   ├── desktop/            # Tauri 2 Desktop Agent (React frontend + Rust backend)
│   └── browser-extension/  # Chrome & Edge Manifest V3 companion extensions
├── packages/
│   ├── shared-types/       # Shared TypeScript types across apps
│   ├── activity-model/     # Application & website activity resolution logic
│   └── config/             # Environment constants & telemetry defaults
├── supabase/
│   ├── migrations/         # PostgreSQL schema & RLS policies
│   ├── functions/          # Edge Functions (if required)
│   └── seed/               # Default database seed data
├── docs/                   # System architecture, telemetry specs, setup guides
└── tests/                  # Integration test suites & test fixtures
```

## Quick Start (Development)

### Prerequisites
- Node.js 22+
- Rust 1.92+
- Visual Studio Build Tools with C++ Workload (Windows)

### Installation & Build

```bash
# Install root monorepo dependencies
npm install

# Build shared packages
npm run build:packages

# Build desktop frontend
npm run build:frontend

# Run Rust test suite
npm run test:rust
```

## Security & Privacy
FocusDrive Desktop Agent strictly collects high-level executable and web domain metadata during active shift hours. It **never** captures screenshots, keystrokes, clipboard data, passwords, or private message content. See [PRIVACY.md](./PRIVACY.md) and [SECURITY.md](./SECURITY.md) for details.
