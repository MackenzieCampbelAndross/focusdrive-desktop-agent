-- ====================================================================
-- FocusDrive Migration: Desktop Agent Specific Access & Telemetry RLS
-- Migration Timestamp: 20260908185500
-- File: 20260908185500_safe_employee_linkage_and_status_alignment.sql
-- ====================================================================

-- 1. Ensure NO global domain CHECK constraint exists on public.employees
-- Existing employees with any domain (e.g. yudhistarsfdc@gmail.com, ranjith@quantumpulse.ltd)
-- remain completely untouched and valid in the FocusDrive database.
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

-- 2. Drop any previous triggers on auth.users to preserve existing Supabase Auth compatibility
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

-- 3. Dedicated Remote Function: Securely Link Verified Desktop User to Existing Active Employee
-- Status semantics rule:
-- The FocusDrive database contains status values such as 'offline', 'online', 'active', etc.
-- An employee with status 'offline' is an authorized employee and MUST NOT be blocked.
-- Only explicitly deactivated/terminated accounts are excluded.
CREATE OR REPLACE FUNCTION public.link_authenticated_desktop_employee()
RETURNS TABLE (
    employee_id UUID,
    employee_name VARCHAR,
    employee_email VARCHAR,
    employee_role VARCHAR,
    employee_department VARCHAR
) AS $$
DECLARE
    v_user_id UUID;
    v_email TEXT;
    v_domain TEXT;
BEGIN
    v_user_id := auth.uid();
    IF v_user_id IS NULL THEN
        RAISE EXCEPTION 'Authentication required.';
    END IF;

    -- Extract verified email from auth JWT
    v_email := LOWER(TRIM(COALESCE(auth.jwt() ->> 'email', '')));
    IF v_email = '' THEN
        RAISE EXCEPTION 'Missing email in authenticated session token.';
    END IF;

    -- Strict domain check for Desktop Agent authentication
    v_domain := split_part(v_email, '@', 2);
    IF v_domain != 'purpledrive.com' THEN
        RAISE EXCEPTION 'Only PurpleDrive company email addresses are authorized to use the Desktop Agent.';
    END IF;

    -- Update and return existing employee row without modifying status or other domain rows
    -- Eligible employees: any employee whose status is not explicitly 'terminated', 'suspended', 'deactivated', or 'inactive'
    RETURN QUERY
    UPDATE public.employees e
    SET auth_user_id = v_user_id,
        updated_at = NOW()
    WHERE LOWER(TRIM(e.email)) = v_email
      AND (e.status IS NULL OR LOWER(TRIM(e.status)) NOT IN ('terminated', 'suspended', 'deactivated', 'inactive'))
      AND (e.auth_user_id IS NULL OR e.auth_user_id = v_user_id)
    RETURNING e.id, e.name, e.email, e.role, e.department;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'No authorized PurpleDrive employee record found for %.', v_email;
    END IF;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

GRANT EXECUTE ON FUNCTION public.link_authenticated_desktop_employee() TO authenticated;

-- 4. Corrective RLS Policies for public.employees
-- Ensures authenticated employees can view their own record by auth_user_id or JWT email
DROP POLICY IF EXISTS "Employees can view own record" ON public.employees;

CREATE POLICY "Employees can view own record" ON public.employees
    FOR SELECT USING (
        (auth.uid() IS NOT NULL AND auth_user_id = auth.uid()) OR
        (auth.jwt() ->> 'email' IS NOT NULL AND LOWER(TRIM(email)) = LOWER(TRIM(auth.jwt() ->> 'email')))
    );

-- 5. Corrective RLS Policies for activity_categories
-- Allow authenticated agents to register newly discovered categories
DROP POLICY IF EXISTS "Employees can insert activity categories" ON public.activity_categories;

CREATE POLICY "Employees can insert activity categories" ON public.activity_categories
    FOR INSERT WITH CHECK (auth.role() = 'authenticated');

-- 6. Helper Function: Check if an email belongs to an authorized PurpleDrive employee
CREATE OR REPLACE FUNCTION public.check_is_authorized_employee(check_email TEXT)
RETURNS BOOLEAN AS $$
DECLARE
    v_clean TEXT;
BEGIN
    v_clean := LOWER(TRIM(check_email));
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

GRANT EXECUTE ON FUNCTION public.check_is_authorized_employee(TEXT) TO anon, authenticated;
