-- Add migration script here
/**
@public
@description
Updates the password for an existing tenant;
Uses upsert_tenant_role to ensure the database role is correctly updated;

@usage
Called from the application layer:
SELECT update_tenant_password('tenant_base', 'existing_tenant', 'new_db_password_plaintext', 'new_db_password_encrypted');

@returns
Returns TRUE if the update was successful, FALSE if the tenant was not found.
*/
CREATE OR REPLACE FUNCTION update_tenant_password(tenant_base_role TEXT, tenant_name TEXT, new_db_password_plaintext TEXT, new_db_password_encrypted TEXT) RETURNS BOOLEAN AS $$
DECLARE
    role_name TEXT;
    hashed_password TEXT;
BEGIN
    -- Check if the tenant exists and get the role name
    SELECT db_user INTO role_name
    FROM tenants
    WHERE name = tenant_name;

    IF role_name IS NULL THEN
        RAISE NOTICE 'Tenant % not found', tenant_name;
        RETURN FALSE;
    END IF;

    -- Update the database role using upsert_tenant_role
    PERFORM upsert_tenant_role(tenant_base_role, role_name, new_db_password_plaintext);

    -- Update the hashed password in the tenants table
    UPDATE tenants
    SET db_password_encrypted = new_db_password_encrypted,
        updated_at = CURRENT_TIMESTAMP
    WHERE name = tenant_name;

    RAISE NOTICE 'Password updated for tenant %', tenant_name;
    RETURN TRUE;
EXCEPTION
    WHEN OTHERS THEN
        RAISE EXCEPTION 'Error updating password for tenant %: %', tenant_name, SQLERRM;
END;
$$ LANGUAGE plpgsql;