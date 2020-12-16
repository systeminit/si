CREATE TABLE organizations (
    id          bigint PRIMARY KEY,
    si_id       text UNIQUE,
    name        text NOT NULL,
    billing_account_id bigint NOT NULL REFERENCES billing_accounts(id),
    tenant_ids  text[] NOT NULL,
    obj         jsonb NOT NULL,
    created_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(name, billing_account_id)
);

CREATE INDEX idx_organizations_tenant_ids ON "organizations" USING GIN ("tenant_ids");

CREATE OR REPLACE FUNCTION organization_create_v1(
    name text, 
    billing_account_id text, 
    OUT object jsonb
) AS 
$$
    DECLARE
        id bigint;
        si_id text;
        billing_account_si_id bigint;
        tenant_ids text[];
        created_at timestamp with time zone;
        updated_at timestamp with time zone;
        si_storable jsonb;
    BEGIN
        SELECT next_si_id_v1() INTO id;
        SELECT 'organization:' || id INTO si_id;
        SELECT split_part(billing_account_id, ':', 2)::bigint INTO billing_account_si_id;
        SELECT ARRAY[billing_account_id, si_id] INTO tenant_ids;
        SELECT NOW() INTO created_at;
        SELECT NOW() INTO updated_at;
        SELECT jsonb_build_object(
                'typeName', 'organization',
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

        INSERT INTO organizations VALUES (
            id,
            si_id,
            name,
            billing_account_si_id, 
            tenant_ids,
            object,
            created_at,
            updated_at
        );
    END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION organization_save_v1(
    organization jsonb,
    OUT object jsonb
) AS 
$$
    DECLARE
        current_organization organizations%rowtype;
        this_id bigint;
        this_tenant_ids text[];
    BEGIN
        /* extract the id */
        SELECT si_id_to_primary_key_v1(organization->>'id') INTO this_id;

        /* fetch the current organization */
        SELECT * INTO current_organization FROM organizations WHERE id = this_id;
        IF NOT FOUND THEN
            RAISE WARNING 'organization id % not found', this_id;
        END IF;

        IF si_id_to_primary_key_v1(organization->'siStorable'->>'billingAccountId') != current_organization.billing_account_id THEN
            RAISE WARNING 'mutated billing account id; not allowed!';
        END IF;

        SELECT ARRAY(SELECT jsonb_array_elements_text(organization->'siStorable'->'tenantIds')) INTO this_tenant_ids;

        UPDATE organizations SET name = organization->>'name', tenant_ids = this_tenant_ids, obj = organization, updated_at = NOW() WHERE id = this_id RETURNING obj INTO object;
    END
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION organization_get_v1(
    si_id text, OUT object jsonb
) AS 
$$
    DECLARE
        this_id bigint;
    BEGIN
        SELECT si_id_to_primary_key_v1(si_id) INTO this_id;
        SELECT w.obj INTO object FROM organizations AS w WHERE id = this_id;
    END
$$ LANGUAGE PLPGSQL STABLE