CREATE OR REPLACE FUNCTION si_storable_create_v1(si_id text, si_workspace_id text,
                                                 this_created_at timestamp with time zone,
                                                 this_updated_at timestamp with time zone,
                                                 OUT our_si_storable jsonb, OUT our_workspace_id bigint,
                                                 OUT our_organization_id bigint, OUT our_billing_account_id bigint,
                                                 OUT our_tenant_ids text[]) AS
$$
DECLARE
    si_billing_account_id text;
    si_organization_id    text;
    this_type_name text;
BEGIN
    SELECT si_id_to_primary_key_v1(si_workspace_id) INTO our_workspace_id;
    SELECT split_part(si_id, ':', 1)::text INTO this_type_name;

    SELECT organization_id, billing_account_id
    INTO our_organization_id, our_billing_account_id
    FROM workspaces
    WHERE id = our_workspace_id;

    SELECT 'billingAccount:' || our_billing_account_id INTO si_billing_account_id;
    SELECT 'organization:' || our_organization_id INTO si_organization_id;

    SELECT ARRAY [
               si_billing_account_id,
               si_organization_id,
               si_workspace_id,
               si_id
               ]
    INTO our_tenant_ids;

    SELECT jsonb_build_object(
                   'typeName', this_type_name,
                   'objectId', si_id,
                   'billingAccountId', si_billing_account_id,
                   'organizationId', si_organization_id,
                   'workspaceId', si_workspace_id,
                   'tenantIds', our_tenant_ids,
                   'deleted', false,
                   'createdAt', this_created_at,
                   'updatedAt', this_updated_at
               )
    INTO our_si_storable;
END ;
$$ LANGUAGE PLPGSQL STABLE;