-- Add migration script here
/**
@public
@description
Retrieves tenant information by UUID.

@usage
Called from the application layer:
SELECT * FROM get_tenant_by_uuid('12345678-1234-1234-1234-123456789012');
*/
CREATE OR REPLACE FUNCTION get_tenant_by_uuid(tenant_id UUID)
    RETURNS TABLE (
                      id UUID,
                      name TEXT,
                      db_user TEXT,
                      db_password_encrypted TEXT,
                      created_at TIMESTAMP WITH TIME ZONE,
                      updated_at TIMESTAMP WITH TIME ZONE
                  ) AS $$
BEGIN
    RETURN QUERY
        SELECT t.id, t.name, t.db_user, t.created_at, t.updated_at
        FROM tenants t
        WHERE t.id = tenant_id;
END;
$$ LANGUAGE plpgsql;