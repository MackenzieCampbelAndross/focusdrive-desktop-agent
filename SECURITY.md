# Security Policy & Architecture

## Security Boundaries

### Native Messaging Payload Validation
All browser extension payloads communicated over Native Messaging are untrusted. The Rust backend validates payload schemas, message length, and origin metadata before processing.

### Supabase Key Policy
- **Client Anon Key Only**: The desktop application frontend ONLY holds public publishable/anon keys.
- **No Service Role Keys**: Service role keys or administrative secrets are NEVER embedded in desktop client code or client builds.

### Query Parameterization
All local SQLite persistence queries use parameterized statements to prevent SQL injection risks.
