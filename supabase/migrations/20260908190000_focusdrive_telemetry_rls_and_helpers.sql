-- ====================================================================
-- FocusDrive Telemetry RLS & Helper Policies
-- Migration Timestamp: 20260908190000
-- File: 20260908190000_focusdrive_telemetry_rls_and_helpers.sql
-- ====================================================================

-- 1. Ensure NO global domain constraint exists on public.employees
-- Existing employee rows (with gmail, quantumpulse, etc.) remain completely untouched.
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_constraint 
        WHERE conname = 'chk_employee_email_purpledrive_domain'
    ) THEN
        ALTER TABLE public.employees
        DROP CONSTRAINT chk_employee_email_purpledrive_domain;
    END IF;
END $$;

-- 2. Drop any previous test triggers on auth.users
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_trigger
        WHERE tgname = 'on_auth_user_created_link_employee'
    ) THEN
        DROP TRIGGER IF EXISTS on_auth_user_created_link_employee ON auth.users;
    END IF;
EXCEPTION
    WHEN OTHERS THEN
        NULL;
END $$;

-- 3. RLS Policies for public.employees using the real schema:
-- Schema: id TEXT, name TEXT, email TEXT, initials TEXT, role TEXT, title TEXT,
--         team_id TEXT, manager_id TEXT, timezone TEXT, shift_start TEXT,
--         shift_end TEXT, shift_overnight BOOLEAN, shift_days TEXT, status TEXT, hired_on DATE
-- Identity mapping: auth.users.email -> public.employees.email
-- Allows authenticated employees to read their own employee profile row.
DO $$
BEGIN
    DROP POLICY IF EXISTS "Employees can view own record" ON public.employees;
    DROP POLICY IF EXISTS "Employees can link own auth user" ON public.employees;

    CREATE POLICY "Employees can view own record" ON public.employees
        FOR SELECT USING (
            auth.jwt() ->> 'email' IS NOT NULL 
            AND LOWER(TRIM(email)) = LOWER(TRIM(auth.jwt() ->> 'email'))
        );
END $$;

-- 4. Activity Categories Registration Policy
-- Allows authenticated Desktop Agents to read & insert newly discovered applications/websites
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.tables 
        WHERE table_schema = 'public' AND table_name = 'activity_categories'
    ) THEN
        DROP POLICY IF EXISTS "Employees can read activity categories" ON public.activity_categories;
        CREATE POLICY "Employees can read activity categories" ON public.activity_categories
            FOR SELECT USING (true);

        DROP POLICY IF EXISTS "Employees can insert activity categories" ON public.activity_categories;
        CREATE POLICY "Employees can insert activity categories" ON public.activity_categories
            FOR INSERT WITH CHECK (auth.role() = 'authenticated');
    END IF;
END $$;

-- 5. Helper Function: Check if an email is authorized to use the Desktop Agent
-- Returns TRUE only if:
--   a) Domain is exactly purpledrive.com
--   b) Exists in public.employees
--   c) Status is not terminated/suspended/inactive/deactivated
CREATE OR REPLACE FUNCTION public.check_is_authorized_desktop_employee(check_email TEXT)
RETURNS BOOLEAN AS $$
DECLARE
    v_clean TEXT;
BEGIN
    v_clean := LOWER(TRIM(check_email));
    -- Exact domain requirement for Desktop Agent access
    IF split_part(v_clean, '@', 2) != 'purpledrive.com' THEN
        RETURN FALSE;
    END IF;

    RETURN EXISTS (
        SELECT 1 FROM public.employees
        WHERE LOWER(TRIM(email)) = v_clean
          AND (status IS NULL OR LOWER(TRIM(status)) NOT IN ('terminated', 'suspended', 'deactivated', 'inactive'))
    );
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

GRANT EXECUTE ON FUNCTION public.check_is_authorized_desktop_employee(TEXT) TO anon, authenticated;
