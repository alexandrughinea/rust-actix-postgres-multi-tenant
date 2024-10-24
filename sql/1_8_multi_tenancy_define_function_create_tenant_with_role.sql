/**
@public
@description
- Function assumes there is a `tenants` table
- We hash the password before storing it in the `tenants` table.
- We still use the plain password for the database role, as PostgreSQL roles require plain-text passwords which is generally acceptable because PostgreSQL handles role passwords securely.

@usage
Called from the application layer:
SELECT create_tenant_with_role('tenant_base_role', 'new_tenant', 'tenant_base_role', 'db_password_plaintext', 'db_password_encrypted');
*/
CREATE OR REPLACE FUNCTION create_tenant_with_role(tenant_base_role TEXT, tenant_name TEXT, role TEXT, db_password_plaintext TEXT, db_password_encrypted TEXT) RETURNS UUID AS $$
DECLARE
    tenant_id UUID;
    -- Join the base role with the tenant's unique name
    db_user TEXT := tenant_base_role || '_' || tenant_name;
BEGIN    
    -- Create or update the role (note: we're using the plain password for the role)
    PERFORM upsert_tenant_role(tenant_base_role, db_user, db_password_plaintext);
    
     -- Insert or update the tenant in the tenants table
    INSERT INTO tenants (name, db_user, db_password_encrypted, role)
    VALUES (tenant_name, db_user, db_password_encrypted, role)
    ON CONFLICT (name) DO UPDATE
    SET 
        db_user = EXCLUDED.db_user,
        db_password_encrypted = EXCLUDED.db_password_encrypted,
        updated_at = CURRENT_TIMESTAMP  -- Update the timestamp on conflict
    RETURNING id INTO tenant_id;
    
    RETURN tenant_id;
END;
$$ LANGUAGE plpgsql;