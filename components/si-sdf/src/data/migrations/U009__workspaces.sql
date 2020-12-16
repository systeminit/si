CREATE TABLE workspaces (
    id          bigint PRIMARY KEY,
    si_id       text UNIQUE,
    name        text NOT NULL,
    billing_account_id bigint NOT NULL REFERENCES billing_accounts(id),
    organization_id bigint NOT NULL REFERENCES organizations(id),
    tenant_ids  text[] NOT NULL,
    obj         jsonb NOT NULL,
    created_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(name, billing_account_id, organization_id)
);

CREATE INDEX idx_workspaces_tenant_ids ON "workspaces" USING GIN ("tenant_ids");

CREATE OR REPLACE FUNCTION workspace_create_v1(
    name text, 
    billing_account_id text, 
    organization_id text, 
    OUT object jsonb
) AS 
$$
    DECLARE
        id bigint;
        si_id text;
        billing_account_si_id bigint;
        organization_si_id bigint;
        tenant_ids text[];
        created_at timestamp with time zone;
        updated_at timestamp with time zone;
        si_storable jsonb;
    BEGIN
        SELECT next_si_id_v1() INTO id;
        SELECT 'workspace:' || id INTO si_id;
        SELECT split_part(billing_account_id, ':', 2)::bigint INTO billing_account_si_id;
        SELECT split_part(organization_id, ':', 2)::bigint INTO organization_si_id;
        SELECT ARRAY[billing_account_id, organization_id, si_id] INTO tenant_ids;
        SELECT NOW() INTO created_at;
        SELECT NOW() INTO updated_at;
        SELECT jsonb_build_object(
                'typeName', 'workspace',
                'tenantIds', tenant_ids,
                'objectId', si_id,
                'billingAccountId', billing_account_id,
                'deleted', false,
                'createdAt', created_at,
                'updatedAt', updated_at
            ) INTO si_storable;

        SELECT jsonb_build_object(
            'id', si_id, 
            'name', name,
            'siStorable', si_storable
        ) INTO object;

        INSERT INTO workspaces VALUES (
            id,
            si_id,
            name,
            billing_account_si_id, 
            organization_si_id,
            tenant_ids,
            object,
            created_at,
            updated_at
        );

        INSERT INTO update_clocks VALUES (DEFAULT, si_id, DEFAULT, DEFAULT);
    END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION workspace_save_v1(
    workspace jsonb,
    OUT object jsonb
) AS 
$$
    DECLARE
        current_workspace workspaces%rowtype; 
        this_id bigint;
        this_tenant_ids text[];
    BEGIN
        /* extract the id */
        SELECT si_id_to_primary_key_v1(workspace->>'id') INTO this_id;

        /* fetch the current workspace */
        SELECT * INTO current_workspace FROM workspaces WHERE id = this_id;
        IF NOT FOUND THEN
            RAISE WARNING 'workspace id % not found', this_id;
        END IF;

        IF si_id_to_primary_key_v1(workspace->'siStorable'->>'billingAccountId') != current_workspace.billing_account_id THEN
            RAISE WARNING 'mutated billing account id; not allowed!';
        END IF;

        IF si_id_to_primary_key_v1(workspace->'siStorable'->>'organizationId') != current_workspace.organization_id THEN
            RAISE WARNING 'mutated organization id; not allowed!';
        END IF;

        SELECT ARRAY(SELECT jsonb_array_elements_text(workspace->'siStorable'->'tenantIds')) INTO this_tenant_ids;

        UPDATE workspaces SET name = workspace->>'name', tenant_ids = this_tenant_ids, obj = workspace, updated_at = NOW() WHERE id = this_id RETURNING obj INTO object;
    END
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION workspace_get_v1(
    si_id text, OUT object jsonb
) AS 
$$
    DECLARE
        this_id bigint;
    BEGIN
        SELECT si_id_to_primary_key_v1(si_id) INTO this_id;
        SELECT w.obj INTO object FROM workspaces AS w WHERE id = this_id;
    END
$$ LANGUAGE PLPGSQL STABLE