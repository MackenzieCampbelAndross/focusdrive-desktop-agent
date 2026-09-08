# Supabase Setup Guide

## Overview
FocusDrive Desktop Agent integrates with Supabase PostgreSQL and Supabase Auth. Development does not require live production credentials during initial setup—mock development mode handles local telemetry persistence in SQLite.

## Environment Configuration
Copy `.env.example` to `.env` in the root workspace directory:
```bash
cp .env.example .env
```

Set the following environment variables when integrating with a Supabase project:
- `VITE_SUPABASE_URL`: Your Supabase project API URL.
- `VITE_SUPABASE_ANON_KEY`: Your Supabase anon (public) key.

## Database Migrations
Migrations are managed in `./supabase/migrations/`.
To apply initial migrations locally using Supabase CLI:
```bash
supabase db push
```

## Row Level Security (RLS)
RLS policies in `supabase/migrations/20260908000000_initial_schema.sql` restrict telemetry rows so employees can only access their own work sessions, break sessions, and activity usage metrics.
