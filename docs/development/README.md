# Development Guide & Testing

## Development Authentication
Phase 1 includes a development authentication mode allowing test employee login without live Supabase credentials:
- **Test Employee 1**: `alex@focusdrive.dev` (Senior Engineer [DEV])
- **Test Employee 2**: `sarah@focusdrive.dev` (Product Manager [DEV])

Password accepts any string in Development Mode.

## Testing Phase 1 Session Flow
```bash
# Run Rust unit tests (state machine, session lifecycle, crash recovery)
npm run test:rust

# Build all packages & desktop frontend
npm run build
```
