CREATE SEQUENCE key_pair_creation_seq;

CREATE TABLE key_pairs
(
    id                 bigint PRIMARY KEY,
    si_id              text UNIQUE,
    name               text                     NOT NULL,
    billing_account_id bigint                   NOT NULL REFERENCES billing_accounts(id),
    tenant_ids         text[]                   NOT NULL,
    obj                jsonb                    NOT NULL,
    created_lamport_clock bigint UNIQUE DEFAULT nextval('key_pair_creation_seq'),
    created_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_key_pairs_tenant_ids ON "key_pairs" USING GIN ("tenant_ids");

CREATE OR REPLACE FUNCTION key_pair_create_v1(
    name text,
    billing_account_id text,
    public_key text,
    secret_key text,
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
        SELECT 'keyPair:' || id INTO si_id;
        SELECT split_part(billing_account_id, ':', 2)::bigint INTO billing_account_si_id;
        SELECT ARRAY[billing_account_id, si_id] INTO tenant_ids;
        SELECT NOW() INTO created_at;
        SELECT NOW() INTO updated_at;
        SELECT jsonb_build_object(
                'typeName', 'keyPair',
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
            'publicKey', public_key,
            'secretKey', secret_key,
            'siStorable', si_storable
        ) INTO object;

        INSERT INTO key_pairs VALUES (
            id,
            si_id,
            name,
            billing_account_si_id,
            tenant_ids,
            object,
            DEFAULT,
            created_at,
            updated_at
        );
    END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION key_pair_get_v1(
    si_id text,
    si_billing_account_id text,
    OUT object jsonb
) AS
$$
    DECLARE
        this_id bigint;
        this_billing_account_id bigint;
    BEGIN
        SELECT si_id_to_primary_key_v1(si_id) INTO this_id;
        SELECT si_id_to_primary_key_v1(si_billing_account_id) INTO this_billing_account_id;

        SELECT o.obj INTO object FROM key_pairs AS o WHERE id = this_id AND billing_account_id = this_billing_account_id;
    END
$$ LANGUAGE PLPGSQL STABLE