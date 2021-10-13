CREATE TABLE schemas
(
    id                 bigint PRIMARY KEY,
    si_id              text UNIQUE,
    name               text   NOT NULL,
    entity_type        text   NOT NULL,
    namespace          text   NOT NULL,
    description        text   NOT NULL,
    billing_account_id bigint REFERENCES billing_accounts (id),
    organization_id    bigint REFERENCES organizations (id),
    workspace_id       bigint REFERENCES workspaces (id),
    tenant_ids         text[] NOT NULL,
    UNIQUE (namespace, name),
    UNIQUE (namespace, entity_type)
);

CREATE TABLE schema_objects
(
    id              bigserial PRIMARY KEY,
    schema_si_id    text                     NOT NULL REFERENCES schemas (si_id),
    schema_id       bigint                   NOT NULL REFERENCES schemas (id),
    change_set_id   bigint REFERENCES change_sets (id),
    edit_session_id bigint REFERENCES edit_sessions (id),
    obj             jsonb                    NOT NULL,
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE schema_variants
(
    id                 bigint PRIMARY KEY,
    si_id              text UNIQUE,
    schema_id          bigint                   NOT NULL references schemas (id),
    name               text                     NOT NULL,
    billing_account_id bigint REFERENCES billing_accounts (id),
    organization_id    bigint REFERENCES organizations (id),
    workspace_id       bigint REFERENCES workspaces (id),
    tenant_ids         text[]                   NOT NULL,
    created_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

ALTER TABLE schema_variants
    ADD CONSTRAINT variant_unique_per_schema UNIQUE (schema_id, name);

CREATE TABLE schema_variants_head
(
    id                   bigint PRIMARY KEY REFERENCES schema_variants (id),
    obj                  jsonb                    NOT NULL,
    root_prop_variant_id bigint,
    tenant_ids           text[]                   NOT NULL,
    created_at           TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE schema_variants_change_set_projection
(
    id                   bigint PRIMARY KEY REFERENCES schema_variants (id),
    obj                  jsonb                    NOT NULL,
    root_prop_variant_id bigint,
    tenant_ids           text[]                   NOT NULL,
    change_set_id        bigint                   NOT NULL REFERENCES change_sets (id),
    created_at           TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (id, change_set_id)
);

CREATE TABLE schema_variants_edit_session_projection
(
    id                   bigint PRIMARY KEY REFERENCES schema_variants (id),
    obj                  jsonb                    NOT NULL,
    root_prop_variant_id bigint,
    tenant_ids           text[]                   NOT NULL,
    change_set_id        bigint                   NOT NULL REFERENCES change_sets (id),
    edit_session_id      bigint                   NOT NULL REFERENCES edit_sessions (id),
    created_at           TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (id, change_set_id, edit_session_id)
);

CREATE OR REPLACE FUNCTION schema_create_v1(
    namespace text,
    name text,
    entity_type text,
    description text,
    this_billing_account_si_id text,
    this_organization_si_id text,
    this_workspace_si_id text,
    this_change_set_si_id text,
    this_edit_session_si_id text,
    OUT object jsonb
) AS
$$
DECLARE
    id                      bigint;
    this_organization_id    bigint;
    this_billing_account_id bigint;
    this_workspace_id       bigint;
    this_change_set_id      bigint;
    this_edit_session_id    bigint;
    tenant_ids              text[];
    si_id                   text;
    created_at              timestamp with time zone;
    updated_at              timestamp with time zone;
    si_storable             jsonb;
BEGIN
    SELECT next_si_id_v1() INTO id;
    SELECT 'schema:' || id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT si_id_to_primary_key_v1(this_change_set_si_id) INTO this_change_set_id;
    SELECT si_id_to_primary_key_v1(this_edit_session_si_id) INTO this_edit_session_id;

    SELECT our_si_storable, our_workspace_id, our_organization_id, our_billing_account_id, our_tenant_ids
    INTO si_storable, this_organization_id, this_billing_account_id, this_workspace_id, tenant_ids
    FROM global_storable_create_v1(si_id, this_workspace_si_id, this_organization_si_id, this_billing_account_si_id,
                                   created_at, updated_at);

    SELECT jsonb_build_object(
                   'id', si_id,
                   'name', name,
                   'namespace', namespace,
                   'description', description,
                   'entityType', entity_type,
                   'siStorable', si_storable
               )
    INTO object;

    INSERT INTO schemas (id, si_id, name, entity_type, namespace, description, billing_account_id, organization_id,
                         workspace_id, tenant_ids, created_at, updated_at)
    VALUES (id, si_id, name, entity_type, namespace, description, this_billing_account_id, this_organization_id,
            this_workspace_id,
            tenant_ids, created_at, updated_at);

    INSERT INTO schemas_edit_session_projection (id, obj, tenant_ids, change_set_id, edit_session_id, created_at,
                                                 updated_at)
    VALUES (id, object, tenant_ids, this_change_set_id, this_edit_session_id, created_at, updated_at);
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION schema_create_global_v1(
    namespace text,
    name text,
    entity_type text,
    description text,
    OUT object jsonb
) AS
$$
DECLARE
    id                      bigint;
    this_organization_id    bigint;
    this_billing_account_id bigint;
    this_workspace_id       bigint;
    tenant_ids              text[];
    si_id                   text;
    created_at              timestamp with time zone;
    updated_at              timestamp with time zone;
    si_storable             jsonb;
BEGIN
    SELECT next_si_id_v1() INTO id;
    SELECT 'schema:' || id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT our_si_storable, our_workspace_id, our_organization_id, our_billing_account_id, our_tenant_ids
    INTO si_storable, this_organization_id, this_billing_account_id, this_workspace_id, tenant_ids
    FROM global_storable_create_v1(si_id, null, null, null,
                                   created_at, updated_at);

    SELECT jsonb_build_object(
                   'id', si_id,
                   'name', name,
                   'namespace', namespace,
                   'description', description,
                   'entityType', entity_type,
                   'siStorable', si_storable
               )
    INTO object;

    INSERT INTO schemas (id, si_id, name, entity_type, namespace, description, billing_account_id, organization_id,
                         workspace_id, tenant_ids, created_at, updated_at)
    VALUES (id, si_id, name, entity_type, namespace, description, this_billing_account_id, this_organization_id,
            this_workspace_id,
            tenant_ids, created_at, updated_at);

    INSERT INTO schemas_head (id, obj, tenant_ids, created_at, updated_at)
    VALUES (id, object, tenant_ids, created_at, updated_at);
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION schema_variant_create_v1(
    this_schema_si_id text,
    this_name text,
    this_description text,
    this_change_set_si_id text,
    this_edit_session_si_id text,
    OUT object jsonb
) AS
$$
DECLARE
    id                         bigint;
    this_schema_id             bigint;
    this_organization_si_id    text;
    this_organization_id       bigint;
    this_billing_account_si_id text;
    this_billing_account_id    bigint;
    this_workspace_si_id       text;
    this_workspace_id          bigint;
    this_change_set_id         bigint;
    this_edit_session_id       bigint;
    tenant_ids                 text[];
    si_id                      text;
    created_at                 timestamp with time zone;
    updated_at                 timestamp with time zone;
    si_storable                jsonb;
    this_schema                schemas%ROWTYPE;
BEGIN
    SELECT next_si_id_v1() INTO id;
    SELECT 'schemaVariant:' || id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT si_id_to_primary_key_v1(this_schema_si_id) INTO this_schema_id;

    SELECT * FROM schemas WHERE schemas.si_id = this_schema_si_id INTO this_schema;

    IF this_schema.workspace_id IS NOT NULL AND this_schema.organization_id IS NOT NULL AND
       this_schema.billing_account_id IS NOT NULL THEN
        SELECT 'workspace:' || this_schema.workspace_id INTO this_workspace_si_id;
        SELECT this_schema.workspace_id INTO this_workspace_id;
        SELECT 'organization:' || this_schema.organization_id INTO this_organization_si_id;
        SELECT this_schema.organization_id INTO this_organization_id;
        SELECT 'billingAccount:' || this_schema.billing_account_id INTO this_billing_account_si_id;
        SELECT this_schema.billing_account_id INTO this_billing_account_id;
    END IF;

    SELECT si_id_to_primary_key_v1(this_change_set_si_id) INTO this_change_set_id;
    SELECT si_id_to_primary_key_v1(this_edit_session_si_id) INTO this_edit_session_id;

    SELECT our_si_storable, our_workspace_id, our_organization_id, our_billing_account_id, our_tenant_ids
    INTO si_storable, this_organization_id, this_billing_account_id, this_workspace_id, tenant_ids
    FROM global_storable_create_v1(si_id, this_workspace_si_id, this_organization_si_id, this_billing_account_si_id,
                                   created_at, updated_at);

    SELECT jsonb_build_object(
                   'id', si_id,
                   'name', this_name,
                   'schemaId', this_schema_si_id,
                   'description', this_description,
                   'siStorable', si_storable
               )
    INTO object;

    INSERT INTO schema_variants (id, si_id, schema_id, name, tenant_ids, billing_account_id, organization_id,
                                 workspace_id,
                                 created_at, updated_at)
    VALUES (id, si_id, this_schema_id, this_name, this_schema.tenant_ids, this_billing_account_id, this_organization_id,
            this_workspace_id,
            created_at, updated_at);

    INSERT INTO schema_variants_edit_session_projection (id, obj, tenant_ids, change_set_id, edit_session_id,
                                                         created_at,
                                                         updated_at)
    VALUES (id, object, this_schema.tenant_ids, this_change_set_id, this_edit_session_id, created_at, updated_at);
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION schema_variant_create_global_v1(
    this_schema_si_id text,
    this_name text,
    this_description text,
    OUT object jsonb
) AS
$$
DECLARE
    id                         bigint;
    this_schema_id             bigint;
    this_organization_si_id    text;
    this_organization_id       bigint;
    this_billing_account_si_id text;
    this_billing_account_id    bigint;
    this_workspace_si_id       text;
    this_workspace_id          bigint;
    tenant_ids                 text[];
    si_id                      text;
    created_at                 timestamp with time zone;
    updated_at                 timestamp with time zone;
    si_storable                jsonb;
    this_schema                schemas%ROWTYPE;
BEGIN
    SELECT next_si_id_v1() INTO id;
    SELECT 'schemaVariant:' || id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT si_id_to_primary_key_v1(this_schema_si_id) INTO this_schema_id;

    SELECT * FROM schemas WHERE schemas.si_id = this_schema_si_id INTO this_schema;

    SELECT our_si_storable, our_workspace_id, our_organization_id, our_billing_account_id, our_tenant_ids
    INTO si_storable, this_organization_id, this_billing_account_id, this_workspace_id, tenant_ids
    FROM global_storable_create_v1(si_id, this_workspace_si_id, this_organization_si_id, this_billing_account_si_id,
                                   created_at, updated_at);

    SELECT jsonb_build_object(
                   'id', si_id,
                   'name', this_name,
                   'schemaId', this_schema_si_id,
                   'description', this_description,
                   'siStorable', si_storable
               )
    INTO object;

    INSERT INTO schema_variants (id, si_id, schema_id, name, tenant_ids, billing_account_id, organization_id,
                                 workspace_id,
                                 created_at, updated_at)
    VALUES (id, si_id, this_schema_id, this_name, this_schema.tenant_ids, this_billing_account_id, this_organization_id,
            this_workspace_id,
            created_at, updated_at);

    INSERT INTO schema_variants_head (id, obj, tenant_ids, created_at, updated_at)
    VALUES (id, object, this_schema.tenant_ids, created_at, updated_at);
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION schema_variant_save_for_edit_session_v1(this_schema_variant_obj jsonb,
                                                                   this_change_set_si_id text,
                                                                   this_edit_session_si_id text
) RETURNS VOID AS
$$
DECLARE
    this_id                      bigint;
    this_tenant_ids              text[];
    this_change_set_id           bigint;
    this_edit_session_id         bigint;
    this_root_prop_variant_si_id text;
    this_root_prop_variant_id    bigint;
BEGIN
    SELECT si_id_to_primary_key_v1(this_schema_variant_obj ->> 'id') INTO this_id;
    SELECT si_id_to_primary_key_v1(this_change_set_si_id) INTO this_change_set_id;
    SELECT si_id_to_primary_key_v1(this_edit_session_si_id) INTO this_edit_session_id;

    IF this_schema_variant_obj ? 'rootPropVariantId' THEN
        SELECT this_schema_variant_obj ->> 'rootPropVariantId' INTO this_root_prop_variant_si_id;
        SELECT si_id_to_primary_key_or_null_v1(this_root_prop_variant_si_id) INTO this_root_prop_variant_id;
    END IF;

    SELECT tenant_ids FROM schema_variants WHERE id = this_id INTO this_tenant_ids;

    INSERT INTO schema_variants_edit_session_projection (id, root_prop_variant_id, obj, change_set_id, edit_session_id,
                                                         tenant_ids, created_at, updated_at)
    VALUES (this_id,
            this_root_prop_variant_id,
            this_schema_variant_obj,
            this_change_set_id,
            this_edit_session_id,
            this_tenant_ids,
            DEFAULT,
            DEFAULT)
    ON CONFLICT (id, change_set_id, edit_session_id) DO UPDATE SET obj                  = this_schema_variant_obj,
                                                                   root_prop_variant_id = this_root_prop_variant_id,
                                                                   updated_at           = now();
END
$$ LANGUAGE PLPGSQL VOLATILE;

