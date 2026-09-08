-- ====================================================================
-- FocusDrive Migration: Organization Email Domain Restriction & Auth Mapping
-- Migration Timestamp: 20260908183000
-- File: 20260908183000_auth_domain_restriction_and_mapping.sql
-- ====================================================================

-- 1. Hard Domain Restriction Constraint on public.employees
-- Ensures that any employee inserted or updated MUST possess an exact @purpledrive.com email.
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint 
        WHERE conname = 'chk_employee_email_purpledrive_domain'
    ) THEN
        ALTER TABLE public.employees
        ADD CONSTRAINT chk_employee_email_purpledrive_domain
        CHECK (email ~* '^[A-Za-z0-9._%+-]+@purpledrive\.com$');
    END IF;
END $$;

-- 2. Trigger Function: Automatically Link Authenticated Supabase User to Existing Employee Record
-- When an employee verifies their OTP, an auth.users record is created or signed in.
-- This trigger binds auth.users.id -> public.employees.auth_user_id without duplicating records.
CREATE OR REPLACE FUNCTION public.handle_auth_user_link_employee()
RETURNS TRIGGER AS $$
DECLARE
    v_clean_email TEXT;
BEGIN
    v_clean_email := LOWER(TRIM(NEW.email));

    -- Enforce exact @purpledrive.com email domain on auth signups
    IF v_clean_email NOT LIKE '%@purpledrive.com' OR split_part(v_clean_email, '@', 2) != 'purpledrive.com' THEN
        RAISE EXCEPTION 'Access restricted: Only @purpledrive.com work email addresses are permitted.';
    END IF;

    -- Link existing employee record if present
    UPDATE public.employees
    SET auth_user_id = NEW.id,
        updated_at = NOW()
    WHERE LOWER(TRIM(email)) = v_clean_email
      AND (auth_user_id IS NULL OR auth_user_id = NEW.id);

    RETURN NEW;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Safe creation of trigger on auth.users (if accessible in Supabase instance)
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.tables 
        WHERE table_schema = 'auth' AND table_name = 'users'
    ) THEN
        DROP TRIGGER IF EXISTS on_auth_user_created_link_employee ON auth.users;
        CREATE TRIGGER on_auth_user_created_link_employee
            AFTER INSERT OR UPDATE OF email ON auth.users
            FOR EACH ROW
            EXECUTE FUNCTION public.handle_auth_user_link_employee();
    END IF;
EXCEPTION
    WHEN insufficient_privilege THEN
        RAISE NOTICE 'Skipping direct trigger on auth.users due to cloud permissions; auth hook or post-login mapping will manage linkage.';
END $$;

-- 3. RLS Update: Allow Authenticated Users to Read Their Own Linked or Pending Employee Profile
-- During initial OTP sign-in, auth_user_id may be newly set, or the client needs to query by email or auth.uid()
DROP POLICY IF EXISTS "Employees can view own record" ON public.employees;

CREATE POLICY "Employees can view own record" ON public.employees
    FOR SELECT USING (
        (auth.uid() IS NOT NULL AND auth_user_id = auth.uid()) OR
        (auth.jwt() ->> 'email' IS NOT NULL AND LOWER(TRIM(email)) = LOWER(TRIM(auth.jwt() ->> 'email')))
    );

-- 4. RLS Policy: Employees can update their own auth_user_id link if unclaimed
DROP POLICY IF EXISTS "Employees can link own auth user" ON public.employees;

CREATE POLICY "Employees can link own auth user" ON public.employees
    FOR UPDATE USING (
        (auth.uid() IS NOT NULL AND auth_user_id = auth.uid()) OR
        (auth.uid() IS NOT NULL AND auth_user_id IS NULL AND LOWER(TRIM(email)) = LOWER(TRIM(auth.jwt() ->> 'email')))
    )
    WITH CHECK (
        auth.uid() IS NOT NULL AND auth_user_id = auth.uid()
    );

-- 5. RLS Policy for Activity Categories Upsert
-- Allows desktop agents to register or upsert newly detected standard activity categories
DROP POLICY IF EXISTS "Employees can insert activity categories" ON public.activity_categories;

CREATE POLICY "Employees can insert activity categories" ON public.activity_categories
    FOR INSERT WITH CHECK (auth.role() = 'authenticated');

-- 6. Helper Function: Verify Employee Organization Status
CREATE OR REPLACE FUNCTION public.check_is_authorized_employee(check_email TEXT)
RETURNS BOOLEAN AS $$
BEGIN
    RETURN EXISTS (
        SELECT 1 FROM public.employees
        WHERE LOWER(TRIM(email)) = LOWER(TRIM(check_email))
          AND split_part(LOWER(TRIM(email)), '@', 2) = 'purpledrive.com'
          AND status = 'active'
    );
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;
