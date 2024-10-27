/**
@private
@description
- Looks up the matching tenant id against the `current_user` (of the connected session pool user);
*/
CREATE OR REPLACE FUNCTION set_tenant_id() RETURNS TRIGGER AS $$
BEGIN
    RAISE NOTICE 'set_tenant_id trigger fired for table: %, operation: %', TG_TABLE_NAME, TG_OP;
    RAISE NOTICE 'Current user: %, Current role: %', current_user, current_setting('role');

    IF NEW.tenant_id IS NULL THEN
        SELECT id INTO NEW.tenant_id
        FROM tenants
        WHERE db_user = current_user;
        
        IF NEW.tenant_id IS NULL THEN
            RAISE EXCEPTION 'No tenant found for user: %', current_user;
        END IF;
        
        RAISE NOTICE 'Setting tenant_id to % for user %', NEW.tenant_id, current_user;
    ELSE
        RAISE NOTICE 'Tenant ID already set: %', NEW.tenant_id;
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;