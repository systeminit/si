CREATE TABLE entities
(
    id                 bigint PRIMARY KEY,
    si_id              text UNIQUE,
    entity_type        text,
    schema_id          bigint                   NOT NULL REFERENCES schemas (id),
    billing_account_id bigint                   NOT NULL REFERENCES billing_accounts (id),
    organization_id    bigint                   NOT NULL REFERENCES organizations (id),
    workspace_id       bigint                   NOT NULL REFERENCES workspaces (id),
    node_id            bigint                   NOT NULL REFERENCES nodes (id),
    tenant_ids         text[]                   NOT NULL,
    created_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_entities_tenant_ids ON "entities" USING GIN ("tenant_ids");

CREATE TABLE entities_head
(
    id         bigint PRIMARY KEY REFERENCES entities (id),
    obj        jsonb                    NOT NULL,
    tenant_ids text[]                   NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE entities_change_set_projection
(
    id            bigint REFERENCES entities (id),
    obj           jsonb                    NOT NULL,
    change_set_id bigint                   NOT NULL REFERENCES change_sets (id),
    tenant_ids    text[]                   NOT NULL,
    created_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (id, change_set_id)
);

CREATE TABLE entities_edit_session_projection
(
    id              bigint REFERENCES entities (id),
    obj             jsonb                    NOT NULL,
    change_set_id   bigint                   NOT NULL REFERENCES change_sets (id),
    edit_session_id bigint                   NOT NULL REFERENCES edit_sessions (id),
    tenant_ids      text[]                   NOT NULL,
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (id, change_set_id, edit_session_id)
);

CREATE OR REPLACE FUNCTION entity_create_v1(this_name text,
                                            this_description text,
                                            this_entity_type text,
                                            this_node_si_id text,
                                            this_change_set_si_id text,
                                            this_edit_session_si_id text,
                                            si_workspace_id text,
                                            OUT object jsonb) AS
$$
DECLARE
    this_id                 bigint;
    si_id                   text;
    this_workspace_id       bigint;
    this_organization_id    bigint;
    this_billing_account_id bigint;
    this_change_set_id      bigint;
    this_edit_session_id    bigint;
    this_node_id            bigint;
    tenant_ids              text[];
    created_at              timestamp with time zone;
    updated_at              timestamp with time zone;
    si_storable             jsonb;
    si_change_set           jsonb;
    this_properties         jsonb;
    this_array_meta         jsonb;
    this_tombstones         jsonb;
    this_ops                jsonb;
    this_schema_id          bigint;
    this_schema_si_id       text;
BEGIN
    SELECT next_si_id_v1() INTO this_id;
    SELECT 'entity:' || this_id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT our_si_storable, our_organization_id, our_billing_account_id, our_workspace_id, our_tenant_ids
    INTO si_storable, this_organization_id, this_billing_account_id, this_workspace_id, tenant_ids
    FROM si_storable_create_v1(si_id, si_workspace_id, created_at, updated_at);

    SELECT si_id_to_primary_key_v1(this_change_set_si_id) INTO this_change_set_id;
    SELECT si_id_to_primary_key_v1(this_edit_session_si_id) INTO this_edit_session_id;
    SELECT si_id_to_primary_key_v1(this_node_si_id) INTO this_node_id;

    SELECT jsonb_build_object('changeSetId', this_change_set_si_id,
                              'editSessionId', this_edit_session_si_id)
    INTO si_change_set;

    SELECT '{}'::jsonb INTO this_properties;
    SELECT '{}'::jsonb INTO this_array_meta;
    SELECT '[]'::jsonb INTO this_ops;
    SELECT '[]'::jsonb INTO this_tombstones;

    /* WARNING: This should be removed. It will dynamically create missing
       schemas, to keep things working. It needs to become an error!
     */
    SELECT id FROM schemas WHERE entity_type = this_entity_type INTO this_schema_id;
    RAISE WARNING 'Have an existing schema: %', this_schema_id;
    IF this_schema_id IS NULL THEN
        SELECT si_id_to_primary_key_v1(schema_create_v1(this_entity_type, this_entity_type, this_entity_type) ->> 'id')
        INTO this_schema_id;
        RAISE WARNING 'Created a schema: %', this_schema_id;
    END IF;
    SELECT 'schema:' || this_schema_id INTO this_schema_si_id;
    /* WARNING OVER - BUT FOR REAL, DROP THIS EVENTUALLY */

    SELECT jsonb_build_object(
                   'id', si_id,
                   'nodeId', this_node_si_id,
                   'name', this_name,
                   'description', this_description,
                   'entityType', this_entity_type,
                   'schemaId', this_schema_si_id,
                   'ops', this_ops,
                   'tombstones', this_tombstones,
                   'arrayMeta', this_array_meta,
                   'properties', this_properties,
                   'siStorable', si_storable,
                   'siChangeSet', si_change_set
               )
    INTO object;

    INSERT INTO entities (id, si_id, entity_type, schema_id, billing_account_id, organization_id, workspace_id, node_id,
                          tenant_ids, created_at,
                          updated_at)
    VALUES (this_id, si_id, this_entity_type, this_schema_id, this_billing_account_id, this_organization_id, this_workspace_id,
            this_node_id, tenant_ids, created_at, updated_at);

    INSERT INTO entities_edit_session_projection (id, obj, change_set_id, edit_session_id, tenant_ids, created_at,
                                                  updated_at)
    VALUES (this_id, object, this_change_set_id, this_edit_session_id, tenant_ids, created_at, updated_at);
END ;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION entity_save_for_edit_session_v1(input_entity jsonb,
                                                           change_set_si_id text,
                                                           edit_session_si_id text
) RETURNS VOID AS
$$
DECLARE
    this_id              bigint;
    this_tenant_ids      text[];
    this_change_set_id   bigint;
    this_edit_session_id bigint;
BEGIN
    SELECT si_id_to_primary_key_v1(input_entity ->> 'id') INTO this_id;
    SELECT si_id_to_primary_key_v1(change_set_si_id) INTO this_change_set_id;
    SELECT si_id_to_primary_key_v1(edit_session_si_id) INTO this_edit_session_id;

    SELECT tenant_ids FROM entities WHERE id = this_id INTO this_tenant_ids;

    INSERT INTO entities_edit_session_projection (id, obj, change_set_id, edit_session_id, tenant_ids,
                                                  created_at, updated_at)
    VALUES (this_id,
            input_entity,
            this_change_set_id,
            this_edit_session_id,
            this_tenant_ids,
            DEFAULT,
            DEFAULT)
    ON CONFLICT (id, change_set_id, edit_session_id) DO UPDATE SET obj        = input_entity,

                                                                   updated_at = now();
END
$$ LANGUAGE PLPGSQL VOLATILE;

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