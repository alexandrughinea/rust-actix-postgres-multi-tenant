-- Add migration script here
DO $$
    DECLARE
        tenant_base_role TEXT := 'tenant_base';
        security_policy_impacted_tables TEXT[] := ARRAY['users'];
        current_table TEXT;
    BEGIN
        -- Setup DB level role and access policies:
        PERFORM setup_tenant_base_role(tenant_base_role);

        -- Apply multi-tenant setup:
        FOREACH current_table IN ARRAY security_policy_impacted_tables
            LOOP
                PERFORM add_tenant_id_column(current_table);
                PERFORM add_current_user_tenant_rls_policy(current_table, tenant_base_role);

                PERFORM add_before_upsert_set_tenant_id_trigger(current_table);
            END LOOP;
    EXCEPTION
        WHEN OTHERS THEN
            RAISE NOTICE 'An error occurred: % %', SQLERRM, SQLSTATE;
    END $$;