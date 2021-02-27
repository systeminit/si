CREATE TABLE billing_accounts (
    id          bigint PRIMARY KEY,
    si_id       text UNIQUE,
    name        text NOT NULL UNIQUE,
    tenant_ids  text[] NOT NULL,
    obj         jsonb NOT NULL,
    created_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_billing_accounts_tenant_ids ON "billing_accounts" USING GIN ("tenant_ids");

CREATE OR REPLACE FUNCTION billing_account_create_v1(
    name text,
    description text,
    OUT object jsonb
) AS
$$
    DECLARE
        id bigint;
        si_id text;
        tenant_ids text[];
        created_at timestamp with time zone;
        updated_at timestamp with time zone;
        si_storable jsonb;
    BEGIN
        SELECT next_si_id_v1() INTO id;
        SELECT 'billingAccount:' || id INTO si_id;
        SELECT ARRAY[si_id] INTO tenant_ids;
        SELECT NOW() INTO created_at;
        SELECT NOW() INTO updated_at;
        SELECT jsonb_build_object(
                'typeName', 'billingAccount',
                'tenantIds', tenant_ids,
                'objectId', si_id,
                'billingAccountId', si_id,
                'deleted', false,
                'createdAt', created_at,
                'updatedAt', updated_at
            ) INTO si_storable;

        SELECT jsonb_build_object(
            'id', si_id,
            'name', name,
            'description', description,
            'siStorable', si_storable
        ) INTO object;

        INSERT INTO billing_accounts VALUES (
            id,
            si_id,
            name,
            tenant_ids,
            object,
            created_at,
            updated_at
        );
    END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION billing_account_save_v1(
    billing_account jsonb,
    OUT object jsonb
) AS
$$
    DECLARE
        current_billing_account billing_accounts%rowtype;
        this_id bigint;
        this_tenant_ids text[];
    BEGIN
        /* extract the id */
        SELECT si_id_to_primary_key_v1(billing_account->>'id') INTO this_id;

        /* fetch the current billing_account */
        SELECT * INTO current_billing_account FROM billing_accounts WHERE id = this_id;
        IF NOT FOUND THEN
            RAISE WARNING 'billing_account id % not found', this_id;
        END IF;

        IF si_id_to_primary_key_v1(billing_account->'siStorable'->>'billingAccountId') != current_billing_account.billing_account_id THEN
            RAISE WARNING 'mutated billing account id; not allowed!';
        END IF;

        SELECT ARRAY(SELECT jsonb_array_elements_text(billing_account->'siStorable'->'tenantIds')) INTO this_tenant_ids;

        UPDATE billing_accounts SET name = billing_account->>'name', tenant_ids = this_tenant_ids, obj = billing_account, updated_at = NOW() WHERE id = this_id RETURNING obj INTO object;
    END
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION billing_account_get_v1(
    si_id text, OUT object jsonb
) AS
$$
    DECLARE
        this_id bigint;
    BEGIN
        SELECT si_id_to_primary_key_v1(si_id) INTO this_id;
        SELECT w.obj INTO object FROM billing_accounts AS w WHERE id = this_id;
    END
$$ LANGUAGE PLPGSQL STABLE