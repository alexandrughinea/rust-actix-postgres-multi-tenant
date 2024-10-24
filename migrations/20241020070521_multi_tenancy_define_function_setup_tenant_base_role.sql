-- Add migration script here
/**
@private
@description
- Accepts an optional excluded_tables parameter, which defaults to an empty array if not provided;
- It iterates over the `excluded_tables` array and revokes all privileges on each specified table;
- It checks if each `excluded table` exists before attempting to revoke privileges, providing a warning for non-existent tables;
- The `tenants` table is `always excluded`;
- The function remains idempotent and can be safely run multiple times;

@usage
1. Without excluded tables:
SELECT setup_tenant_base_role_and_access('tenant_base');

2. With excluded tables:
SELECT setup_tenant_base_role_and_access('tenant_base', ARRAY['example_table1', 'example_table2']);
*/
CREATE OR REPLACE FUNCTION setup_tenant_base_role(
    tenant_base_role TEXT,
    excluded_tables TEXT[] DEFAULT '{}'::TEXT[]
)
    RETURNS VOID AS $$
DECLARE
    table_name TEXT;
BEGIN
    -- Create the tenant_base role if it doesn't exist
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = tenant_base_role) THEN
        EXECUTE format('CREATE ROLE %I NOINHERIT', tenant_base_role);
        RAISE NOTICE 'Created new role: %', tenant_base_role;
    ELSE
        RAISE NOTICE 'Role % already exists', tenant_base_role;
    END IF;

    -- Grant usage on the public schema
    EXECUTE format('GRANT USAGE ON SCHEMA public TO %I', tenant_base_role);

    -- Grant CRUD access to tenants for all tables in public schema
    FOR table_name IN (SELECT tablename FROM pg_tables WHERE schemaname = 'public')
        LOOP
            EXECUTE format('GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE %I TO %I', table_name, tenant_base_role);
        END LOOP;

    -- Revoke privileges on excluded tables
    FOREACH table_name IN ARRAY excluded_tables
        LOOP
            IF EXISTS (SELECT FROM pg_tables WHERE schemaname = 'public' AND tablename = table_name) THEN
                EXECUTE format('REVOKE ALL PRIVILEGES ON TABLE %I FROM %I', table_name, tenant_base_role);
                RAISE NOTICE 'Revoked privileges on table % from role %', table_name, tenant_base_role;
            ELSE
                RAISE WARNING 'Table % does not exist in public schema, skipping', table_name;
            END IF;
        END LOOP;

    -- Add limitations for tenants table (always excluded)
    EXECUTE format('REVOKE ALL PRIVILEGES ON TABLE tenants FROM %I', tenant_base_role);

    -- Allow limited select on tenants (for querying the tenant id)
    EXECUTE format('GRANT SELECT (id, db_user) ON TABLE tenants TO %I', tenant_base_role);

    -- Grant usage on sequences
    EXECUTE format('GRANT USAGE ON ALL SEQUENCES IN SCHEMA public TO %I', tenant_base_role);

    RAISE NOTICE 'Setup completed successfully for role: %', tenant_base_role;
EXCEPTION
    WHEN OTHERS THEN
        RAISE EXCEPTION 'Error in setup: %', SQLERRM;
END;
$$ LANGUAGE plpgsql;