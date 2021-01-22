CREATE TABLE entities
(
    id                 bigint PRIMARY KEY,
    si_id              text UNIQUE,
    object_type        text,
    billing_account_id bigint                   NOT NULL REFERENCES billing_accounts (id),
    organization_id    bigint                   NOT NULL REFERENCES organizations (id),
    workspace_id       bigint                   NOT NULL REFERENCES workspaces (id),
    node_id            bigint                   NOT NULL REFERENCES nodes (id),
    tenant_ids         text[]                   NOT NULL,
    created_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_entities_tenant_ids ON "entities" USING GIN ("tenant_ids");

CREATE TABLE entities_base
(
    id            bigint PRIMARY KEY REFERENCES entities (id),
    obj           jsonb                    NOT NULL,
    change_set_id bigint                   NOT NULL REFERENCES change_sets (id),
    change_set_epoch        bigint                   NOT NULL,
    change_set_update_count bigint                   NOT NULL,
    tenant_ids    text[]                   NOT NULL,
    created_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE entities_head
(
    id           bigint PRIMARY KEY REFERENCES entities (id),
    obj          jsonb                    NOT NULL,
    tenant_ids   text[]                   NOT NULL,
    epoch        bigint                   NOT NULL,
    update_count bigint                   NOT NULL,
    created_at   TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE entities_projection
(
    id                      bigint REFERENCES entities (id),
    obj                     jsonb                    NOT NULL,
    change_set_id           bigint                   NOT NULL REFERENCES change_sets (id),
    change_set_epoch        bigint                   NOT NULL,
    change_set_update_count bigint                   NOT NULL,
    epoch                   bigint                   NOT NULL,
    update_count            bigint                   NOT NULL,
    tenant_ids              text[]                   NOT NULL,
    created_at              TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (id, change_set_id)
);

CREATE OR REPLACE FUNCTION entity_create_v1(this_name text,
                                            this_description text,
                                            this_object_type text,
                                            this_node_si_id text,
                                            this_change_set_si_id text,
                                            this_edit_session_si_id text,
                                            this_si_change_set_event text,
                                            si_workspace_id text,
                                            this_workspace_epoch bigint,
                                            this_workspace_update_count bigint,
                                            this_change_set_epoch bigint,
                                            this_change_set_update_count bigint,
                                            OUT object jsonb) AS
$$
DECLARE
    this_id                    bigint;
    si_id                      text;
    this_workspace_id          bigint;
    this_organization_id       bigint;
    this_billing_account_id    bigint;
    this_change_set_id         bigint;
    this_node_id               bigint;
    tenant_ids                 text[];
    created_at                 timestamp with time zone;
    updated_at                 timestamp with time zone;
    si_storable                jsonb;
    si_change_set              jsonb;
    si_change_set_update_clock jsonb;
    base_object                jsonb;
    this_properties            jsonb;
BEGIN
    SELECT next_si_id_v1() INTO this_id;
    SELECT 'entity:' || this_id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT our_si_storable, our_organization_id, our_billing_account_id, our_workspace_id, our_tenant_ids
    INTO si_storable, this_organization_id, this_billing_account_id, this_workspace_id, tenant_ids
    FROM si_storable_create_v1(si_id, si_workspace_id, created_at, updated_at, this_workspace_epoch,
                               this_workspace_update_count);

    SELECT si_id_to_primary_key_v1(this_change_set_si_id) INTO this_change_set_id;
    SELECT si_id_to_primary_key_v1(this_node_si_id) INTO this_node_id;

    SELECT jsonb_build_object(
                   'epoch', this_change_set_epoch,
                   'updateCount', this_change_set_update_count
               )
    INTO si_change_set_update_clock;

    SELECT jsonb_build_object('changeSetId', this_change_set_si_id,
                              'editSessionId', this_edit_session_si_id,
                              'event', this_si_change_set_event,
                              'orderClock', si_change_set_update_clock
               )
    INTO si_change_set;

    SELECT jsonb_build_object('__baseline', '{}'::jsonb) INTO this_properties;

    SELECT jsonb_build_object(
                   'id', si_id,
                   'name', this_name,
                   'objectType', this_object_type,
                   'description', this_description,
                   'nodeId', this_node_si_id,
                   'expressionProperties', this_properties,
                   'manualProperties', this_properties,
                   'inferredProperties', this_properties,
                   'properties', this_properties,
                   'head', false,
                   'base', false,
                   'siChangeSet', si_change_set,
                   'siStorable', si_storable
               )
    INTO object;

    SELECT jsonb_build_object(
                   'id', si_id,
                   'name', this_name,
                   'objectType', this_object_type,
                   'description', this_description,
                   'nodeId', this_node_si_id,
                   'expressionProperties', this_properties,
                   'manualProperties', this_properties,
                   'inferredProperties', this_properties,
                   'properties', this_properties,
                   'head', false,
                   'base', true,
                   'siChangeSet', si_change_set,
                   'siStorable', si_storable
               )
    INTO base_object;

    INSERT INTO entities (id, si_id, object_type, billing_account_id, organization_id, workspace_id, node_id,
                          tenant_ids, created_at,
                          updated_at)
    VALUES (this_id, si_id, this_object_type, this_billing_account_id, this_organization_id, this_workspace_id,
            this_node_id, tenant_ids,
            created_at, updated_at);

    INSERT INTO entities_projection (id, obj, change_set_id, change_set_epoch, change_set_update_count, epoch,
                                     update_count, tenant_ids, created_at, updated_at)
    VALUES (this_id, object, this_change_set_id, this_change_set_epoch, this_change_set_update_count,
            this_workspace_epoch, this_workspace_update_count, tenant_ids, created_at, updated_at);

    INSERT INTO entities_base (id, obj, change_set_id, change_set_epoch, change_set_update_count, tenant_ids, created_at, updated_at)
    VALUES (this_id, base_object, this_change_set_id, this_change_set_epoch, this_change_set_update_count, tenant_ids, created_at, updated_at);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION entity_save_projection_v1(input_entity jsonb,
                                                     OUT object jsonb) AS
$$
DECLARE
    this_id         bigint;
    this_tenant_ids text[];
BEGIN
    /* extract the id */
    SELECT si_id_to_primary_key_v1(input_entity ->> 'id') INTO this_id;

    SELECT tenant_ids FROM entities WHERE id = this_id INTO this_tenant_ids;

    INSERT INTO entities_projection (id, obj, change_set_id, change_set_epoch, change_set_update_count, epoch,
                                     update_count, tenant_ids, created_at, updated_at)
    VALUES (this_id,
            input_entity,
            si_id_to_primary_key_v1(input_entity -> 'siChangeSet' ->> 'changeSetId'),
            (input_entity -> 'siChangeSet' -> 'orderClock' ->> 'epoch')::bigint,
            (input_entity -> 'siChangeSet' -> 'orderClock' ->> 'updateCount')::bigint,
            (input_entity -> 'siStorable' -> 'updateClock' ->> 'epoch')::bigint,
            (input_entity -> 'siStorable' -> 'updateClock' ->> 'updateCount')::bigint,
            this_tenant_ids,
            DEFAULT,
            DEFAULT)
    ON CONFLICT (id, change_set_id) DO UPDATE SET obj                     = input_entity,
                                                  change_set_epoch        = (input_entity -> 'siChangeSet' -> 'orderClock' ->> 'epoch')::bigint,
                                                  change_set_update_count = (input_entity -> 'siChangeSet' -> 'orderClock' ->> 'updateCount')::bigint,
                                                  epoch                   = (input_entity -> 'siStorable' -> 'updateClock' ->> 'epoch')::bigint,
                                                  update_count            = (input_entity -> 'siStorable' -> 'updateClock' ->> 'updateCount')::bigint,
                                                  updated_at              = now()
    RETURNING obj INTO object;
END
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION entity_save_base_v1(input_entity jsonb,
                                               OUT object jsonb) AS
$$
DECLARE
    this_id         bigint;
    this_tenant_ids text[];
BEGIN
    /* extract the id */
    SELECT si_id_to_primary_key_v1(input_entity ->> 'id') INTO this_id;

    SELECT tenant_ids FROM entities WHERE id = this_id INTO this_tenant_ids;

    INSERT INTO entities_base (id, obj, change_set_id, change_set_epoch, change_set_update_count, tenant_ids, created_at, updated_at)
    VALUES (this_id,
            input_entity,
            si_id_to_primary_key_v1(input_entity -> 'siChangeSet' ->> 'changeSetId'),
            (input_entity -> 'siChangeSet' -> 'orderClock' ->> 'epoch')::bigint,
            (input_entity -> 'siChangeSet' -> 'orderClock' ->> 'updateCount')::bigint,
            this_tenant_ids,
            DEFAULT,
            DEFAULT)
    ON CONFLICT (id) DO UPDATE SET obj        = input_entity,
                                   change_set_epoch        = (input_entity -> 'siChangeSet' -> 'orderClock' ->> 'epoch')::bigint,
                                   change_set_update_count = (input_entity -> 'siChangeSet' -> 'orderClock' ->> 'updateCount')::bigint,
                                   updated_at = now()
    RETURNING obj INTO object;
END
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION entity_save_head_v1(input_entity jsonb,
                                               OUT object jsonb) AS
$$
DECLARE
    this_id         bigint;
    this_tenant_ids text[];
BEGIN
    /* extract the id */
    SELECT si_id_to_primary_key_v1(input_entity ->> 'id') INTO this_id;

    SELECT tenant_ids FROM entities WHERE id = this_id INTO this_tenant_ids;

    INSERT INTO entities_head (id, obj, epoch, update_count, tenant_ids, created_at, updated_at)
    VALUES (this_id, input_entity, (input_entity -> 'siStorable' -> 'updateClock' ->> 'epoch')::bigint,
            (input_entity -> 'siStorable' -> 'updateClock' ->> 'updateCount')::bigint, this_tenant_ids, DEFAULT,
            DEFAULT)
    ON CONFLICT (id) DO UPDATE SET obj          = input_entity,
                                   epoch        = (input_entity -> 'siStorable' -> 'updateClock' ->> 'epoch')::bigint,
                                   update_count = (input_entity -> 'siStorable' -> 'updateClock' ->> 'updateCount')::bigint,
                                   updated_at   = now()
    RETURNING obj INTO object;

    DELETE FROM entities_base WHERE id = this_id;
END
$$ LANGUAGE PLPGSQL;