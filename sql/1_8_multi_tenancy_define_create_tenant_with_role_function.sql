/**
@public
@description
- Function assumes there is a `tenants` table
- We hash the password before storing it in the `tenants` table.
- We still use the plain password for the database role, as PostgreSQL roles require plain-text passwords which is generally acceptable because PostgreSQL handles role passwords securely.

@usage
Called from the application layer:
SELECT create_tenant_with_role('tenant_base_role', 'new_tenant', 'db_password_plaintext', 'db_password_encrypted');
*/
CREATE OR REPLACE FUNCTION create_tenant_with_role(tenant_base_role TEXT, tenant_name TEXT, db_password_plaintext TEXT, db_password_encrypted TEXT) RETURNS UUID AS $$
DECLARE
    tenant_id UUID;
    -- Join the base role with the tenant's unique name
    role_name TEXT := tenant_base_role || '_' || tenant_name;
BEGIN    
    -- Create or update the role (note: we're using the plain password for the role)
    PERFORM upsert_tenant_role(tenant_base_role, role_name, db_password_plaintext);
    
    -- Insert or update the tenant in the tenants table with the hashed password
    INSERT INTO tenants (name, db_user, db_password_encrypted)
    VALUES (tenant_name, role_name, db_password_encrypted)
    ON CONFLICT (name) DO UPDATE
    SET db_user = EXCLUDED.db_user, db_password_encrypted = EXCLUDED.db_password_encrypted
    RETURNING id INTO tenant_id;
    
    RETURN tenant_id;
END;
$$ LANGUAGE plpgsql;