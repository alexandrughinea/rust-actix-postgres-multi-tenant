-- Add migration script here
/**
@private
@description
Adds a `tenant_id` column name to the specified table.
*/
CREATE OR REPLACE FUNCTION add_tenant_id_column(table_name TEXT) RETURNS void AS $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND information_schema.columns.table_name = add_tenant_id_column.table_name
          AND column_name = 'tenant_id'
    ) THEN
        EXECUTE format('
            ALTER TABLE %I
            ADD COLUMN tenant_id UUID REFERENCES tenants(id)
        ', table_name);
        RAISE NOTICE 'Added tenant_id column to table: %', table_name;
    ELSE
        RAISE NOTICE 'tenant_id column already exists in table: %', table_name;
    END IF;
EXCEPTION
    WHEN undefined_table THEN
        RAISE EXCEPTION 'Table % does not exist', table_name;
    WHEN OTHERS THEN
        RAISE EXCEPTION 'Error adding tenant_id column to table %: %', table_name, SQLERRM;
END;
$$ LANGUAGE plpgsql;