/**
@private
@description
For each INSERT OR UPDATE we set the tenant id automatically for a table.
*/
CREATE OR REPLACE FUNCTION add_before_upsert_set_tenant_id_trigger(table_name TEXT) RETURNS void AS $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 
        FROM information_schema.tables 
        WHERE table_schema = 'public' 
          AND information_schema.tables.table_name = add_before_upsert_set_tenant_id_trigger.table_name
    ) THEN
        RAISE NOTICE 'Table % does not exist, skipping trigger creation', table_name;
        RETURN;
    END IF;

    IF NOT EXISTS (
        SELECT 1 
        FROM information_schema.triggers 
        WHERE trigger_name = 'before_upsert_set_tenant_id_trigger' 
          AND event_object_table = add_before_upsert_set_tenant_id_trigger.table_name
    ) THEN
        EXECUTE format('
            CREATE TRIGGER before_upsert_set_tenant_id_trigger
            BEFORE INSERT OR UPDATE ON %I
            FOR EACH ROW
            EXECUTE FUNCTION set_tenant_id();
        ', table_name);
        RAISE NOTICE 'Added set_tenant_id trigger to table: %', table_name;
    ELSE
        RAISE NOTICE 'Trigger before_upsert_set_tenant_id_trigger already exists on table: %', table_name;
    END IF;
END;
$$ LANGUAGE plpgsql;