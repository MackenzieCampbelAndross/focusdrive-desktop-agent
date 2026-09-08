-- Initial PostgreSQL Schema & RLS for FocusDrive Telemetry Layer
-- Migration: 20260908000000_initial_schema.sql

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Employees Table
CREATE TABLE IF NOT EXISTS public.employees (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    auth_user_id UUID REFERENCES auth.users(id) ON DELETE SET NULL,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'employee',
    department VARCHAR(100),
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Devices Table
CREATE TABLE IF NOT EXISTS public.devices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL REFERENCES public.employees(id) ON DELETE CASCADE,
    device_name VARCHAR(255) NOT NULL,
    os VARCHAR(100) NOT NULL,
    agent_version VARCHAR(50) NOT NULL,
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Work Sessions Table
CREATE TABLE IF NOT EXISTS public.work_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL REFERENCES public.employees(id) ON DELETE CASCADE,
    device_id UUID NOT NULL REFERENCES public.devices(id) ON DELETE CASCADE,
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Break Sessions Table
CREATE TABLE IF NOT EXISTS public.break_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    work_session_id UUID NOT NULL REFERENCES public.work_sessions(id) ON DELETE CASCADE,
    employee_id UUID NOT NULL REFERENCES public.employees(id) ON DELETE CASCADE,
    type VARCHAR(50) NOT NULL CHECK (type IN ('manual', 'automatic_idle')),
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ,
    duration_seconds INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Activity Categories Table
CREATE TABLE IF NOT EXISTS public.activity_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL CHECK (type IN ('desktop_application', 'browser')),
    identifier VARCHAR(255) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_activity_category UNIQUE (type, identifier)
);

-- Activity Usage Table
CREATE TABLE IF NOT EXISTS public.activity_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    work_session_id UUID NOT NULL REFERENCES public.work_sessions(id) ON DELETE CASCADE,
    employee_id UUID NOT NULL REFERENCES public.employees(id) ON DELETE CASCADE,
    activity_category_id UUID NOT NULL REFERENCES public.activity_categories(id) ON DELETE RESTRICT,
    source VARCHAR(50) NOT NULL CHECK (source IN ('desktop_application', 'browser')),
    duration_seconds INT NOT NULL DEFAULT 0,
    first_seen_at TIMESTAMPTZ NOT NULL,
    last_seen_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_session_activity UNIQUE (work_session_id, activity_category_id, source)
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_work_sessions_employee ON public.work_sessions(employee_id);
CREATE INDEX IF NOT EXISTS idx_break_sessions_work_session ON public.break_sessions(work_session_id);
CREATE INDEX IF NOT EXISTS idx_activity_usage_work_session ON public.activity_usage(work_session_id);

-- Enable Row Level Security (RLS) on all tables
ALTER TABLE public.employees ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.devices ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.work_sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.break_sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.activity_categories ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.activity_usage ENABLE ROW LEVEL SECURITY;

-- Basic RLS Policies (Employees can read and write their own telemetry)
CREATE POLICY "Employees can view own record" ON public.employees
    FOR SELECT USING (auth.uid() = auth_user_id);

CREATE POLICY "Employees can view own devices" ON public.devices
    FOR SELECT USING (employee_id IN (SELECT id FROM public.employees WHERE auth_user_id = auth.uid()));

CREATE POLICY "Employees can insert own devices" ON public.devices
    FOR INSERT WITH CHECK (employee_id IN (SELECT id FROM public.employees WHERE auth_user_id = auth.uid()));

CREATE POLICY "Employees can manage own work sessions" ON public.work_sessions
    FOR ALL USING (employee_id IN (SELECT id FROM public.employees WHERE auth_user_id = auth.uid()));

CREATE POLICY "Employees can manage own break sessions" ON public.break_sessions
    FOR ALL USING (employee_id IN (SELECT id FROM public.employees WHERE auth_user_id = auth.uid()));

CREATE POLICY "Employees can read activity categories" ON public.activity_categories
    FOR SELECT USING (true);

CREATE POLICY "Employees can manage own activity usage" ON public.activity_usage
    FOR ALL USING (employee_id IN (SELECT id FROM public.employees WHERE auth_user_id = auth.uid()));
