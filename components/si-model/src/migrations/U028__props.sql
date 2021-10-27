CREATE TABLE props
(
    id                 bigint PRIMARY KEY,
    si_id              text UNIQUE,
    name               text                     NOT NULL,
    kind               text                     NOT NULL,
    billing_account_id bigint REFERENCES billing_accounts (id),
    organization_id    bigint REFERENCES organizations (id),
    workspace_id       bigint REFERENCES workspaces (id),
    tenant_ids         text[]                   NOT NULL,
    created_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE props_head
(
    id         bigint PRIMARY KEY REFERENCES props (id),
    obj        jsonb                    NOT NULL,
    tenant_ids text[]                   NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE props_change_set_projection
(
    id            bigint PRIMARY KEY REFERENCES props (id),
    obj           jsonb                    NOT NULL,
    tenant_ids    text[]                   NOT NULL,
    change_set_id bigint                   NOT NULL REFERENCES change_sets (id),
    created_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (id, change_set_id)
);

CREATE TABLE props_edit_session_projection
(
    id              bigint PRIMARY KEY REFERENCES props (id),
    obj             jsonb                    NOT NULL,
    tenant_ids      text[]                   NOT NULL,
    change_set_id   bigint                   NOT NULL REFERENCES change_sets (id),
    edit_session_id bigint                   NOT NULL REFERENCES edit_sessions (id),
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (id, change_set_id)
);

ALTER TABLE props
    ADD CONSTRAINT valid_kind_check CHECK (kind IN ('string', 'number', 'boolean', 'object', 'map', 'array'));

CREATE TABLE prop_variants
(
    id                 bigint PRIMARY KEY,
    si_id              text UNIQUE,
    prop_id            bigint                   NOT NULL references props (id),
    name               text                     NOT NULL,
    billing_account_id bigint REFERENCES billing_accounts (id),
    organization_id    bigint REFERENCES organizations (id),
    workspace_id       bigint REFERENCES workspaces (id),
    tenant_ids         text[]                   NOT NULL,
    created_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

ALTER TABLE prop_variants
    ADD CONSTRAINT variant_unique_per_prop UNIQUE (prop_id, name);

CREATE TABLE prop_variants_head
(
    id         bigint PRIMARY KEY REFERENCES prop_variants (id),
    obj        jsonb                    NOT NULL,
    tenant_ids text[]                   NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE prop_variants_change_set_projection
(
    id            bigint PRIMARY KEY REFERENCES prop_variants (id),
    obj           jsonb                    NOT NULL,
    tenant_ids    text[]                   NOT NULL,
    change_set_id bigint                   NOT NULL REFERENCES change_sets (id),
    created_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (id, change_set_id)
);

CREATE TABLE prop_variants_edit_session_projection
(
    id              bigint PRIMARY KEY REFERENCES prop_variants (id),
    obj             jsonb                    NOT NULL,
    tenant_ids      text[]                   NOT NULL,
    change_set_id   bigint                   NOT NULL REFERENCES change_sets (id),
    edit_session_id bigint                   NOT NULL REFERENCES edit_sessions (id),
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (id, change_set_id)
);

CREATE TABLE prop_variants_schema_variants
(
    prop_variant_id   bigint NOT NULL REFERENCES prop_variants (id),
    schema_variant_id bigint NOT NULL REFERENCES schema_variants (id),
    change_set_id     bigint REFERENCES change_sets (id),
    edit_session_id   bigint REFERENCES edit_sessions (id),
    deleted           bool DEFAULT false
);

ALTER TABLE schema_variants_edit_session_projection
    ADD CONSTRAINT root_prop_variant_id_fk FOREIGN KEY (root_prop_variant_id) REFERENCES prop_variants (id);
ALTER TABLE schema_variants_change_set_projection
    ADD CONSTRAINT root_prop_variant_id_fk FOREIGN KEY (root_prop_variant_id) REFERENCES prop_variants (id);
ALTER TABLE schema_variants_head
    ADD CONSTRAINT root_prop_variant_id_fk FOREIGN KEY (root_prop_variant_id) REFERENCES prop_variants (id);

CREATE OR REPLACE FUNCTION prop_create_v1(
    namespace text,
    name text,
    description text,
    kind text,
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
    SELECT 'prop:' || id INTO si_id;
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
                   'kind', kind,
                   'siStorable', si_storable
               )
    INTO object;

    INSERT INTO props (id, si_id, name, kind, billing_account_id, organization_id, workspace_id, tenant_ids, created_at,
                       updated_at)
    VALUES (id, si_id, name, kind, this_billing_account_id, this_organization_id, this_workspace_id, tenant_ids,
            created_at, updated_at);

    INSERT INTO props_edit_session_projection (id, obj, tenant_ids, change_set_id, edit_session_id, created_at,
                                               updated_at)
    VALUES (id, object, tenant_ids, this_change_set_id, this_edit_session_id, created_at, updated_at);
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION prop_variant_create_v1(
    this_prop_si_id text,
    this_name text,
    this_description text,
    this_change_set_si_id text,
    this_edit_session_si_id text,
    OUT object jsonb
) AS
$$
DECLARE
    id                         bigint;
    this_prop_id               bigint;
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
    this_prop                  props%ROWTYPE;
BEGIN
    SELECT next_si_id_v1() INTO id;
    SELECT 'propVariant:' || id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT si_id_to_primary_key_v1(this_prop_si_id) INTO this_prop_id;

    SELECT * FROM props WHERE props.si_id = this_prop_si_id INTO this_prop;

    IF this_prop.workspace_id IS NOT NULL AND this_prop.organization_id IS NOT NULL AND
       this_prop.billing_account_id IS NOT NULL THEN
        SELECT 'workspace:' || this_prop.workspace_id INTO this_workspace_si_id;
        SELECT this_prop.workspace_id INTO this_workspace_id;
        SELECT 'organization:' || this_prop.organization_id INTO this_organization_si_id;
        SELECT this_prop.organization_id INTO this_organization_id;
        SELECT 'billingAccount:' || this_prop.billing_account_id INTO this_billing_account_si_id;
        SELECT this_prop.billing_account_id INTO this_billing_account_id;
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
                   'description', this_description,
                   'kind', this_prop.kind,
                   'propId', this_prop_si_id,
                   'siStorable', si_storable
               )
    INTO object;

    INSERT INTO prop_variants (id, si_id, prop_id, name, billing_account_id, organization_id, workspace_id, tenant_ids,
                               created_at, updated_at)
    VALUES (id, si_id, this_prop_id, this_name, this_billing_account_id, this_organization_id,
            this_workspace_id, this_prop.tenant_ids, created_at, updated_at);

    INSERT INTO prop_variants_edit_session_projection (id, obj, tenant_ids, change_set_id, edit_session_id,
                                                       created_at,
                                                       updated_at)
    VALUES (id, object, this_prop.tenant_ids, this_change_set_id, this_edit_session_id, created_at, updated_at);
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION prop_variant_add_to_schema_variant_v1(
    this_prop_variant_si_id text,
    this_schema_variant_si_id text,
    this_change_set_si_id text,
    this_edit_session_si_id text
) RETURNS VOID AS
$$
BEGIN
    INSERT INTO prop_variants_schema_variants (prop_variant_id, schema_variant_id, change_set_id, edit_session_id)
    VALUES (si_id_to_primary_key_v1(this_prop_variant_si_id),
            si_id_to_primary_key_v1(this_schema_variant_si_id),
            si_id_to_primary_key_v1(this_change_set_si_id),
            si_id_to_primary_key_v1(this_edit_session_si_id))
    ON CONFLICT DO NOTHING;
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION prop_variant_remove_from_schema_variant_v1(
    this_prop_variant_si_id text,
    this_schema_variant_si_id text,
    this_change_set_si_id text,
    this_edit_session_si_id text
) RETURNS VOID AS
$$
BEGIN
    UPDATE prop_variants_schema_variants
    SET deleted = true
    WHERE prop_variant_id = si_id_to_primary_key_v1(this_prop_variant_si_id)
      AND schema_variant_id = si_id_to_primary_key_v1(this_schema_variant_si_id)
      AND change_set_id = si_id_to_primary_key_v1(this_change_set_si_id)
      AND edit_session_id = si_id_to_primary_key_v1(this_edit_session_si_id);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE TABLE prop_variant_lineage (
    id                     bigint,
    parent_prop_variant_id bigint NOT NULL REFERENCES prop_variants (id),
    child_prop_variant_id  bigint NOT NULL REFERENCES prop_variants (id),
    change_set_id          bigint REFERENCES change_sets (id),
    edit_session_id        bigint REFERENCES edit_sessions (id),
    deleted                bool DEFAULT false,
    UNIQUE (parent_prop_variant_id, child_prop_variant_id, change_set_id, edit_session_id)
);

ALTER TABLE prop_variant_lineage
    ADD CONSTRAINT no_self_referential_lineage CHECK (parent_prop_variant_id != child_prop_variant_id);

CREATE OR REPLACE FUNCTION prop_variant_add_parent_v1(
    this_child_prop_variant_si_id text,
    this_parent_prop_variant_si_id text,
    this_change_set_si_id text,
    this_edit_session_si_id text
) RETURNS VOID AS
$$
DECLARE
    this_parent_prop_kind text;
    this_si_id            text;
    this_id               bigint;
BEGIN
    SELECT kind
    FROM props
             LEFT JOIN prop_variants ON prop_variants.si_id = this_parent_prop_variant_si_id
    WHERE props.id = prop_variants.prop_id
    LIMIT 1
    INTO this_parent_prop_kind;

    SELECT next_si_id_v1() INTO this_id;
    SELECT 'propVariant:' || this_id INTO this_si_id;

    IF this_parent_prop_kind = 'object' OR this_parent_prop_kind = 'array' THEN
        INSERT INTO prop_variant_lineage (id, child_prop_variant_id, parent_prop_variant_id,
                                          change_set_id, edit_session_id, deleted)
        VALUES (this_id,
                si_id_to_primary_key_v1(this_child_prop_variant_si_id),
                si_id_to_primary_key_v1(this_parent_prop_variant_si_id),
                si_id_to_primary_key_v1(this_change_set_si_id),
                si_id_to_primary_key_v1(this_edit_session_si_id),
                false);
    ELSE
        RAISE EXCEPTION 'Invalid parent prop type %', this_parent_prop_kind USING HINT = 'Must be object or array';
    END IF;
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION prop_variant_remove_parent_v1(
    this_child_prop_variant_si_id text,
    this_parent_prop_variant_si_id text,
    this_change_set_si_id text,
    this_edit_session_si_id text
) RETURNS VOID AS
$$
BEGIN
    UPDATE prop_variant_lineage
    SET deleted = true
    WHERE child_prop_variant_id = si_id_to_primary_key_v1(this_child_prop_variant_si_id)
      AND parent_prop_variant_id = si_id_to_primary_key_v1(this_parent_prop_variant_si_id)
      AND change_set_id = si_id_to_primary_key_v1(this_change_set_si_id)
      AND edit_session_id = si_id_to_primary_key_v1(this_edit_session_si_id);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
