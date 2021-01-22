CREATE TABLE resources
(
    id                 bigint PRIMARY KEY,
    si_id              text UNIQUE,
    billing_account_id bigint                   NOT NULL REFERENCES billing_accounts (id),
    organization_id    bigint                   NOT NULL REFERENCES organizations (id),
    workspace_id       bigint                   NOT NULL REFERENCES workspaces (id),
    system_id          bigint                   NOT NULL,
    node_id            bigint                   UNIQUE NOT NULL,
    entity_id          bigint                   UNIQUE NOT NULL,
    tenant_ids         text[]                   NOT NULL,
    created_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_resources_tenant_ids ON "resources" USING GIN ("tenant_ids");

CREATE TABLE resources_head
(
    id           bigint PRIMARY KEY REFERENCES resources (id),
    obj          jsonb                    NOT NULL,
    tenant_ids   text[]                   NOT NULL,
    epoch        bigint                   NOT NULL,
    update_count bigint                   NOT NULL,
    created_at   TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE resources_projection
(
    id            bigint REFERENCES resources (id),
    obj           jsonb                    NOT NULL,
    change_set_id bigint                   NOT NULL REFERENCES change_sets (id),
    epoch         bigint                   NOT NULL,
    update_count  bigint                   NOT NULL,
    tenant_ids    text[]                   NOT NULL,
    created_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (id, change_set_id)
);

CREATE OR REPLACE FUNCTION resource_create_v1(this_state jsonb,
                                              this_status text,
                                              this_health text,
                                              this_timestamp text,
                                              this_unix_timestamp bigint,
                                              this_system_si_id text,
                                              this_node_si_id text,
                                              this_entity_si_id text,
                                              this_change_set_si_id text,
                                              si_workspace_id text,
                                              this_workspace_epoch bigint,
                                              this_workspace_update_count bigint,
                                              OUT object jsonb) AS
$$
DECLARE
    this_id                 bigint;
    si_id                   text;
    this_workspace_id       bigint;
    this_organization_id    bigint;
    this_billing_account_id bigint;
    this_change_set_id      bigint;
    this_node_id            bigint;
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
    FROM si_storable_create_v1(si_id, si_workspace_id, created_at, updated_at, this_workspace_epoch,
                               this_workspace_update_count);

    SELECT si_id_to_primary_key_v1(this_change_set_si_id) INTO this_change_set_id;
    SELECT si_id_to_primary_key_v1(this_node_si_id) INTO this_node_id;
    SELECT si_id_to_primary_key_v1(this_system_si_id) INTO this_system_id;
    SELECT si_id_to_primary_key_v1(this_entity_si_id) INTO this_entity_id;

    SELECT jsonb_build_object(
                   'id', si_id,
                   'state', this_state,
                   'status', this_status,
                   'health', this_health,
                   'systemId', this_system_si_id,
                   'nodeId', this_node_si_id,
                   'entityId', this_entity_si_id,
                   'unixTimestamp', this_unix_timestamp,
                   'timestamp', this_timestamp,
                   'changeSetId', this_change_set_si_id,
                   'siStorable', si_storable
               )
    INTO object;

    INSERT INTO resources (id, si_id, billing_account_id, organization_id, workspace_id, system_id, node_id, entity_id,
                           tenant_ids, created_at, updated_at)
    VALUES (this_id, si_id, this_billing_account_id, this_organization_id, this_workspace_id, this_system_id,
            this_node_id, this_entity_id, tenant_ids, created_at, updated_at);

    INSERT INTO resources_projection (id, obj, change_set_id, epoch,
                                      update_count, tenant_ids, created_at, updated_at)
    VALUES (this_id, object, this_change_set_id, this_workspace_epoch, this_workspace_update_count, tenant_ids,
            created_at, updated_at);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION resource_save_head_v1(input_resource jsonb,
                                                 OUT object jsonb) AS
$$
DECLARE
    this_id                      bigint;
    this_change_set_si_id        text;
    this_tenant_ids              text[];
    input_resource_no_change_set jsonb;
BEGIN
    /* extract the id */
    SELECT si_id_to_primary_key_v1(input_resource ->> 'id') INTO this_id;

    SELECT tenant_ids FROM resources WHERE id = this_id INTO this_tenant_ids;
    SELECT input_resource ->> 'changeSetId' INTO this_change_set_si_id;

    SELECT input_resource - 'changeSetId' INTO input_resource_no_change_set;

    INSERT INTO resources_head (id, obj, tenant_ids, epoch, update_count, created_at, updated_at)
    VALUES (this_id, input_resource_no_change_set, this_tenant_ids,
            (input_resource_no_change_set -> 'siStorable' -> 'updateClock' ->> 'epoch')::bigint,
            (input_resource_no_change_set -> 'siStorable' -> 'updateClock' ->> 'updateCount')::bigint,
            DEFAULT,
            DEFAULT)
    ON CONFLICT (id) DO UPDATE SET obj          = input_resource_no_change_set,
                                   epoch        = (input_resource_no_change_set -> 'siStorable' -> 'updateClock' ->> 'epoch')::bigint,
                                   update_count = (input_resource_no_change_set -> 'siStorable' -> 'updateClock' ->>
                                                   'updateCount')::bigint,
                                   updated_at   = now()
    RETURNING obj INTO object;

    IF this_change_set_si_id IS NOT NULL THEN
        DELETE
        FROM resources_projection
        WHERE id = this_id
          AND change_set_id = si_id_to_primary_key_v1(this_change_set_si_id);
    END IF;
END
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION resource_save_projection_v1(input_resource jsonb,
                                                       OUT object jsonb) AS
$$
DECLARE
    this_id         bigint;
    this_tenant_ids text[];
BEGIN
    /* extract the id */
    SELECT si_id_to_primary_key_v1(input_resource ->> 'id') INTO this_id;

    SELECT tenant_ids FROM resources WHERE id = this_id INTO this_tenant_ids;

    INSERT INTO resources_projection (id, obj, change_set_id, tenant_ids, epoch, update_count, created_at, updated_at)
    VALUES (this_id, input_resource, si_id_to_primary_key_v1(input_resource ->> 'changeSetId'), this_tenant_ids,
            (input_resource -> 'siStorable' -> 'updateClock' ->> 'epoch')::bigint,
            (input_resource -> 'siStorable' -> 'updateClock' ->> 'updateCount')::bigint,
            DEFAULT,
            DEFAULT)
    ON CONFLICT (id, change_set_id) DO UPDATE SET obj          = input_resource,
                                                               epoch        = (input_resource -> 'siStorable' -> 'updateClock' ->> 'epoch')::bigint,
                                                               update_count = (input_resource -> 'siStorable' -> 'updateClock' ->> 'updateCount')::bigint,
                                                               updated_at   = now()
    RETURNING obj INTO object;
END
$$ LANGUAGE PLPGSQL;
