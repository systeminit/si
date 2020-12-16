CREATE TABLE secrets
(
    id                 bigint PRIMARY KEY,
    si_id              text UNIQUE,
    name               text                     NOT NULL,
    key_pair_id        bigint                   NOT NULL REFERENCES key_pairs (id),
    billing_account_id bigint                   NOT NULL REFERENCES billing_accounts (id),
    organization_id    bigint                   NOT NULL REFERENCES organizations (id),
    workspace_id       bigint                   NOT NULL REFERENCES workspaces (id),
    epoch              bigint                   NOT NULL,
    update_count       bigint                   NOT NULL,
    tenant_ids         text[]                   NOT NULL,
    obj                jsonb                    NOT NULL,
    created_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (name, workspace_id)
);

CREATE INDEX idx_secrets_tenant_ids ON "secrets" USING GIN ("tenant_ids");

CREATE OR REPLACE FUNCTION secret_create_v1(this_name text,
                                            object_type text,
                                            kind text,
                                            crypted text,
                                            si_key_pair_id text,
                                            version text,
                                            algorithm text,
                                            si_workspace_id text,
                                            this_epoch bigint,
                                            this_update_count bigint,
                                            OUT object jsonb) AS
$$
DECLARE
    this_id                 bigint;
    si_id                   text;
    this_workspace_id       bigint;
    this_organization_id    bigint;
    this_billing_account_id bigint;
    this_key_pair_id        bigint;
    tenant_ids              text[];
    created_at              timestamp with time zone;
    updated_at              timestamp with time zone;
    si_storable             jsonb;
BEGIN
    SELECT next_si_id_v1() INTO this_id;
    SELECT 'secret:' || this_id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;
    SELECT si_id_to_primary_key_v1(si_key_pair_id) INTO this_key_pair_id;

    SELECT our_si_storable, our_organization_id, our_billing_account_id, our_workspace_id, our_tenant_ids
    INTO si_storable, this_organization_id, this_billing_account_id, this_workspace_id, tenant_ids
    FROM si_storable_create_v1(si_id, si_workspace_id, created_at, updated_at, this_epoch, this_update_count);

    SELECT jsonb_build_object(
                   'id', si_id,
                   'name', this_name,
                   'objectType', object_type,
                   'kind', kind,
                   'crypted', crypted,
                   'keyPairId', si_key_pair_id,
                   'version', version,
                   'algorithm', algorithm,
                   'siStorable', si_storable
               )
    INTO object;

    INSERT INTO secrets (id, si_id, name, key_pair_id, billing_account_id, organization_id, workspace_id, epoch,
                         update_count, tenant_ids, obj, created_at, updated_at)
    VALUES (this_id,
            si_id,
            this_name,
            this_key_pair_id,
            this_billing_account_id,
            this_organization_id,
            this_workspace_id,
            this_epoch,
            this_update_count,
            tenant_ids,
            object,
            created_at,
            updated_at);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION secret_get_v1(si_id text, OUT object jsonb) AS
$$
DECLARE
    this_id bigint;
BEGIN
    SELECT si_id_to_primary_key_v1(si_id) INTO this_id;
    SELECT w.obj INTO object FROM secrets AS w WHERE id = this_id;
END
$$ LANGUAGE PLPGSQL STABLE;
