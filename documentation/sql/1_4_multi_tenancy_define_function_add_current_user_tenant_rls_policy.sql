/**
@private
@description
- Creates a security policy between a table and a tenant base role (very important distinction: not an extended role);
- Roles which inherit from `tenant_base_role` will automatically inherit this policy;
*/
CREATE OR REPLACE FUNCTION add_current_user_tenant_rls_policy(table_name TEXT, tenant_base_role TEXT) RETURNS void AS $$
DECLARE
    policy_name TEXT;
BEGIN
    policy_name := 'tenant_isolation_policy_' || table_name;
    
    EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', table_name);
    
    EXECUTE format('DROP POLICY IF EXISTS %I ON %I', policy_name, table_name);
  
    EXECUTE format('
    CREATE POLICY %I ON %I
    FOR ALL
    TO %I
    USING (
        tenant_id = (SELECT id FROM tenants WHERE db_user = current_user)
    )
    WITH CHECK (
        tenant_id = (SELECT id FROM tenants WHERE db_user = current_user)
    )
		', policy_name, table_name, tenant_base_role);

    RAISE NOTICE 'Tenant isolation policy applied to table: %', table_name;
EXCEPTION
    WHEN undefined_column THEN
        RAISE EXCEPTION 'Table % does not have a tenant_id column', table_name;
    WHEN undefined_table THEN
        RAISE EXCEPTION 'Table % does not exist', table_name;
    WHEN OTHERS THEN
        RAISE EXCEPTION 'Error applying tenant isolation policy to table %: %', table_name, SQLERRM;
END;
$$ LANGUAGE plpgsql;