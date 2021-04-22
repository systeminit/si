CREATE TABLE resources
(
    id                 bigint PRIMARY KEY,
    si_id              text UNIQUE,
    billing_account_id bigint                   NOT NULL REFERENCES billing_accounts (id),
    organization_id    bigint                   NOT NULL REFERENCES organizations (id),
    workspace_id       bigint                   NOT NULL REFERENCES workspaces (id),
    entity_id          bigint                   NOT NULL REFERENCES entities (id),
    system_id          bigint                   NOT NULL REFERENCES entities (id),
    tenant_ids         text[]                   NOT NULL,
    obj                jsonb                    NOT NULL,
    created_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (entity_id, system_id)
);

CREATE INDEX idx_resources_tenant_ids ON "resources" USING GIN ("tenant_ids");

CREATE OR REPLACE FUNCTION resource_create_v1(this_state jsonb,
                                              this_status text,
                                              this_health text,
                                              this_timestamp text,
                                              this_unix_timestamp bigint,
                                              this_system_si_id text,
                                              this_entity_si_id text,
                                              si_workspace_id text,
                                              OUT object jsonb) AS
$$
DECLARE
    this_id                 bigint;
    si_id                   text;
    this_workspace_id       bigint;
    this_organization_id    bigint;
    this_billing_account_id bigint;
    this_system_id          bigint;
    this_entity_id          bigint;
    tenant_ids              text[];
    created_at              timestamp with time zone;
    updated_at              timestamp with time zone;
    si_storable             jsonb;
BEGIN
    SELECT next_si_id_v1() INTO this_id;
    SELECT 'resource:' || this_id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT our_si_storable, our_organization_id, our_billing_account_id, our_workspace_id, our_tenant_ids
    INTO si_storable, this_organization_id, this_billing_account_id, this_workspace_id, tenant_ids
    FROM si_storable_create_v1(si_id, si_workspace_id, created_at, updated_at);

    SELECT si_id_to_primary_key_v1(this_system_si_id) INTO this_system_id;
    SELECT si_id_to_primary_key_v1(this_entity_si_id) INTO this_entity_id;

    SELECT jsonb_build_object(
                   'id', si_id,
                   'state', this_state,
                   'status', this_status,
                   'health', this_health,
                   'entityId', this_entity_si_id,
                   'systemId', this_system_si_id,
                   'unixTimestamp', this_unix_timestamp,
                   'timestamp', this_timestamp,
                   'siStorable', si_storable
               )
    INTO object;

    INSERT INTO resources (id, si_id, billing_account_id, organization_id, workspace_id, entity_id, system_id,
                           tenant_ids, obj, created_at, updated_at)
    VALUES (this_id, si_id, this_billing_account_id, this_organization_id, this_workspace_id,
            this_entity_id, this_system_id, tenant_ids, object, created_at, updated_at);
END ;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION resource_save_v1(input_resource jsonb,
                                            OUT object jsonb) AS
$$
DECLARE
    this_current resources%rowtype;
    this_id      bigint;
BEGIN
    /* extract the id */
    SELECT si_id_to_primary_key_v1(input_resource ->> 'id') INTO this_id;

    SELECT * INTO this_current FROM resources WHERE id = this_id;
    IF NOT FOUND THEN
        RAISE WARNING 'resource id % not found', this_id;
    END IF;

    /* bail if it is a tenancy violation */
    IF si_id_to_primary_key_v1(input_resource -> 'siStorable' ->> 'billingAccountId') !=
       this_current.billing_account_id THEN
        RAISE WARNING 'mutated billing account id; not allowed!';
    END IF;

    UPDATE resources
    SET obj        = input_resource,
        updated_at = NOW()
    WHERE id = this_id
    RETURNING obj INTO object;
END
$$ LANGUAGE PLPGSQL;