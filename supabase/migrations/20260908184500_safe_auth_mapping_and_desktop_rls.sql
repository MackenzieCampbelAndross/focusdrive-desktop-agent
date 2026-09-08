-- ====================================================================
-- FocusDrive Corrective Migration: Safe Employee Linkage & Desktop RLS
-- Migration Timestamp: 20260908184500
-- File: 20260908184500_safe_auth_mapping_and_desktop_rls.sql
-- ====================================================================

-- 1. Remove global CHECK constraint if previously attempted or present
-- This ensures existing public.employees with other domains are never violated or modified.
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

-- 2. Clean up any previous trigger on auth.users if created
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
        -- Ignore permission errors on auth schema
        NULL;
END $$;

-- 3. Dedicated Remote Function: Securely Link Verified Desktop User to Existing Active Employee
-- This function verifies that:
--   a) The user's JWT / email is strictly an @purpledrive.com address
--   b) An existing active employee record exists matching the verified email
--   c) Updates the employee's auth_user_id without duplicating or altering other employee rows
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
        RAISE EXCEPTION 'Access restricted: FocusDrive Desktop Agent only authorizes @purpledrive.com employees.';
    END IF;

    -- Find existing employee matching this email and active status
    RETURN QUERY
    UPDATE public.employees e
    SET auth_user_id = v_user_id,
        updated_at = NOW()
    WHERE LOWER(TRIM(e.email)) = v_email
      AND e.status = 'active'
      AND (e.auth_user_id IS NULL OR e.auth_user_id = v_user_id)
    RETURNING e.id, e.name, e.email, e.role, e.department;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'No active authorized employee record found for %.', v_email;
    END IF;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Grant execution to authenticated users
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

-- 6. Helper Function: Check if an email belongs to an authorized active PurpleDrive employee
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
          AND status = 'active'
    );
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

GRANT EXECUTE ON FUNCTION public.check_is_authorized_employee(TEXT) TO anon, authenticated;
