-- Add migration script here
/*
@private
@description:
Usually called by another postgres function, it is never called directly from application layer.
1. If the role already exists:

    It updates the password.
    It ensures the role has LOGIN capability.
    It checks if the role inherits from tenant_base, and if not, it grants that inheritance.

2. If the role doesn't exist:
    It creates the role with the specified password, LOGIN capability, and inheriting from `tenant_base_role`.
@usage
Update the password of a certain DB role.
Usually called by `create_tenant_with_role(..)`

Examples:
SELECT upsert_tenant_role('tenant_base', 'tenant_base_alex', 'db_password_plaintext_1');
SELECT upsert_tenant_role('tenant_base', 'tenant_base_bob', 'db_password_plaintext_2');

DO $$
BEGIN
  PERFORM upsert_tenant_role('tenant_base', 'tenant_base_alex', 'db_password_plaintext_1');
  PERFORM upsert_tenant_role('tenant_base', 'tenant_base_bob', 'db_password_plaintext_2');
  -- Add more roles as needed
END $$;
*/
CREATE OR REPLACE FUNCTION upsert_tenant_role(tenant_base_role TEXT, role_name TEXT, db_password_plaintext TEXT) RETURNS void AS $$
BEGIN
    IF EXISTS (SELECT FROM pg_roles WHERE rolname = role_name) THEN
        -- Role exists, update it
        EXECUTE format('ALTER ROLE %I WITH LOGIN PASSWORD %L', role_name, db_password_plaintext);

        -- Ensure it inherits from `tenant_base_role`
        IF NOT EXISTS (
            SELECT FROM pg_auth_members m
                            JOIN pg_roles r ON (m.roleid = r.oid)
            WHERE r.rolname = tenant_base_role AND m.member = (SELECT oid FROM pg_roles WHERE rolname = role_name)
        ) THEN
            EXECUTE format('GRANT %I TO %I', tenant_base_role, role_name);
        END IF;

        RAISE NOTICE 'Role % updated', role_name;
    ELSE
        -- Role doesn't exist, create it
        EXECUTE format('CREATE ROLE %I WITH LOGIN PASSWORD %L IN ROLE %I', role_name, db_password_plaintext, tenant_base_role);
        RAISE NOTICE 'Role % created', role_name;
    END IF;
END;
$$ LANGUAGE plpgsql;

