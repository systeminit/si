CREATE TABLE api_clients
(
    id                 bigint PRIMARY KEY,
    si_id              text UNIQUE,
    name               text                     NOT NULL,
    valid_token_hash   text,
    billing_account_id bigint                   NOT NULL REFERENCES billing_accounts (id),
    tenant_ids         text[]                   NOT NULL,
    obj                jsonb                    NOT NULL,
    created_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (name, billing_account_id)
);

CREATE INDEX idx_api_clients_tenant_ids ON "api_clients" USING GIN ("tenant_ids");

CREATE OR REPLACE FUNCTION api_client_create_v1(name text,
                                                billing_account_id text,
                                                kind text,
                                                OUT object jsonb) AS
$$
DECLARE
    id                    bigint;
    si_id                 text;
    billing_account_si_id bigint;
    tenant_ids            text[];
    created_at            timestamp with time zone;
    updated_at            timestamp with time zone;
    si_storable           jsonb;
BEGIN
    SELECT next_si_id_v1() INTO id;
    SELECT 'apiClient:' || id INTO si_id;
    SELECT split_part(billing_account_id, ':', 2)::bigint INTO billing_account_si_id;
    SELECT ARRAY [billing_account_id, si_id] INTO tenant_ids;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;
    SELECT jsonb_build_object(
                   'typeName', 'apiClient',
                   'tenantIds', tenant_ids,
                   'objectId', si_id,
                   'billingAccountId', billing_account_id,
                   'deleted', false,
                   'createdAt', created_at,
                   'updatedAt', updated_at
               )
    INTO si_storable;

    SELECT jsonb_build_object(
                   'id', si_id,
                   'name', name,
                   'kind', kind,
                   'siStorable', si_storable
               )
    INTO object;

    INSERT INTO api_clients (id,
                             si_id,
                             name,
                             billing_account_id,
                             tenant_ids,
                             obj,
                             created_at,
                             updated_at)
    VALUES (id,
            si_id,
            name,
            billing_account_si_id,
            tenant_ids,
            object,
            created_at,
            updated_at);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION api_client_set_valid_token_hash_v1(si_id text, this_valid_token_hash text) RETURNS void AS
$$
DECLARE
    this_id                    bigint;
BEGIN
    SELECT si_id_to_primary_key_v1(si_id) INTO this_id;
    UPDATE api_clients SET valid_token_hash = this_valid_token_hash WHERE id = this_id;
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION api_client_get_v1(si_id text, OUT object jsonb) AS
$$
DECLARE
    this_id bigint;
BEGIN
    SELECT si_id_to_primary_key_v1(si_id) INTO this_id;
    SELECT w.obj INTO object FROM api_clients AS w WHERE id = this_id;
END
$$ LANGUAGE PLPGSQL STABLE;
