DO $$
DECLARE
    tenant_base_role TEXT := 'tenant_base';
    security_policy_impacted_tables TEXT[] := ARRAY['users'];
    current_table TEXT;
BEGIN
    -- Setup DB level role and access policies:
    PERFORM setup_tenant_base_role(tenant_base_role);

    -- Add two tenants example:
    -- Decryption key used application layer side (AES256bit GCM HEX rep): 4b5d623f8a9b2dc3e78f5c6a1d3b9f0e2a1c4b7d5e8f0a3c6b9d2e5f8a1c4d7b
    PERFORM create_tenant_with_role(tenant_base_role, 'alex', 'secure_password_test_1', '3d0353bd1f90f4e2d6b001d0c5a9cc23fd65a4712f1d8f9452750d46fe36aaeec120ac11889138bb156e731194eca0d9ff59');
    PERFORM create_tenant_with_role(tenant_base_role, 'bob', 'secure_password_test_2', 'bcaa85f753b81a1c693d90866487e85111a08f2525a0d35bed9045436f6b747c50248cebb7b6c70bf9d65624cc0950a890c6');
    PERFORM create_tenant_with_role(tenant_base_role, 'stefan', 'secure_password_test_3', 'f76ab741aa15f105ddafad9fd8b6b9df5d4760108eff821862859e6d4269e4a56e77cf32824f912f3b6bc9aeb48e12f221a4');

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