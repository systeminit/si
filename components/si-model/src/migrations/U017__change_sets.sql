CREATE TABLE change_sets
(
    id                 bigint PRIMARY KEY,
    si_id              text UNIQUE,
    name               text                     NOT NULL,
    billing_account_id bigint                   NOT NULL REFERENCES billing_accounts (id),
    organization_id    bigint                   NOT NULL REFERENCES organizations (id),
    workspace_id       bigint                   NOT NULL REFERENCES workspaces (id),
    tenant_ids         text[]                   NOT NULL,
    obj                jsonb                    NOT NULL,
    created_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_change_sets_tenant_ids ON "change_sets" USING GIN ("tenant_ids");

CREATE OR REPLACE FUNCTION change_set_create_v1(this_name text,
                                                this_note text,
                                                this_status text,
                                                si_workspace_id text,
                                                OUT object jsonb) AS
$$
DECLARE
    this_id                 bigint;
    si_id                   text;
    this_workspace_id       bigint;
    this_organization_id    bigint;
    this_billing_account_id bigint;
    tenant_ids              text[];
    created_at              timestamp with time zone;
    updated_at              timestamp with time zone;
    si_storable             jsonb;
BEGIN
    SELECT next_si_id_v1() INTO this_id;
    SELECT 'changeSet:' || this_id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT our_si_storable, our_organization_id, our_billing_account_id, our_workspace_id, our_tenant_ids
    INTO si_storable, this_organization_id, this_billing_account_id, this_workspace_id, tenant_ids
    FROM si_storable_create_v1(si_id, si_workspace_id, created_at, updated_at);

    SELECT jsonb_build_object(
                   'id', si_id,
                   'name', this_name,
                   'note', this_note,
                   'status', this_status,
                   'siStorable', si_storable
               )
    INTO object;

    INSERT INTO change_sets (id, si_id, name, billing_account_id, organization_id, workspace_id, tenant_ids, obj,
                             created_at, updated_at)
    VALUES (this_id, si_id, this_name, this_billing_account_id, this_organization_id,
            this_workspace_id, tenant_ids, object, created_at, updated_at);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION change_set_save_v1(input_change_set jsonb,
                                              OUT object jsonb) AS
$$
DECLARE
    this_current change_sets%rowtype;
    this_id      bigint;
BEGIN
    /* extract the id */
    SELECT si_id_to_primary_key_v1(input_change_set ->> 'id') INTO this_id;

    SELECT * INTO this_current FROM change_sets WHERE id = this_id;
    IF NOT FOUND THEN
        RAISE WARNING 'change_set id % not found', this_id;
    END IF;

    /* bail if it is a tenancy violation */
    IF si_id_to_primary_key_v1(input_change_set -> 'siStorable' ->> 'billingAccountId') !=
       this_current.billing_account_id THEN
        RAISE WARNING 'mutated billing account id; not allowed!';
    END IF;

    UPDATE change_sets
    SET name       = input_change_set ->> 'name',
        obj        = input_change_set,
        updated_at = NOW()
    WHERE id = this_id
    RETURNING obj INTO object;
END
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION change_set_get_v1(si_id text, OUT object jsonb) AS
$$
DECLARE
    this_id bigint;
BEGIN
    SELECT si_id_to_primary_key_v1(si_id) INTO this_id;
    SELECT w.obj INTO object FROM change_sets AS w WHERE id = this_id;
END
$$ LANGUAGE PLPGSQL STABLE;

CREATE OR REPLACE FUNCTION change_set_apply_v1(this_si_id text, OUT object jsonb) AS
$$
DECLARE
    this_id bigint;
BEGIN
    /* extract the id */
    SELECT si_id_to_primary_key_v1(this_si_id) INTO this_id;

    UPDATE change_sets SET obj = jsonb_set(obj, '{status}', '"applied"'::jsonb) WHERE id = this_id;
    SELECT obj INTO object FROM change_sets WHERE id = this_id;

    INSERT INTO entities_head (id, obj, tenant_ids, created_at)
    SELECT entities_change_set_projection.id,
           entities_change_set_projection.obj,
           entities_change_set_projection.tenant_ids,
           entities_change_set_projection.created_at
    FROM entities_change_set_projection
    WHERE entities_change_set_projection.change_set_id = this_id
      AND entities_change_set_projection.obj -> 'siStorable' -> 'deleted' != 'true'
    ON CONFLICT(id) DO UPDATE
        SET obj        = excluded.obj,
            updated_at = NOW();

    DELETE
    FROM entities_head
    WHERE entities_head.id IN (
        SELECT entities_change_set_projection.id
        FROM entities_change_set_projection
        WHERE entities_change_set_projection.change_set_id = this_id
          AND entities_change_set_projection.obj -> 'siStorable' -> 'deleted' = 'true'
    );

    INSERT INTO schemas_head (id, obj, tenant_ids, created_at)
    SELECT schemas_change_set_projection.id,
           schemas_change_set_projection.obj,
           schemas_change_set_projection.tenant_ids,
           schemas_change_set_projection.created_at
    FROM schemas_change_set_projection
    WHERE schemas_change_set_projection.change_set_id = this_id
      AND schemas_change_set_projection.obj -> 'siStorable' -> 'deleted' != 'true'
    ON CONFLICT(id) DO UPDATE
        SET obj        = excluded.obj,
            updated_at = NOW();

    DELETE
    FROM schemas_head
    WHERE schemas_head.id IN (
        SELECT schemas_change_set_projection.id
        FROM schemas_change_set_projection
        WHERE schemas_change_set_projection.change_set_id = this_id
          AND schemas_change_set_projection.obj -> 'siStorable' -> 'deleted' = 'true'
    );

    INSERT INTO schema_variants_head (id, obj, root_prop_variant_id, tenant_ids, created_at)
    SELECT schema_variants_change_set_projection.id,
           schema_variants_change_set_projection.obj,
           schema_variants_change_set_projection.root_prop_variant_id,
           schema_variants_change_set_projection.tenant_ids,
           schema_variants_change_set_projection.created_at
    FROM schema_variants_change_set_projection
    WHERE schema_variants_change_set_projection.change_set_id = this_id
      AND schema_variants_change_set_projection.obj -> 'siStorable' -> 'deleted' != 'true'
    ON CONFLICT(id) DO UPDATE
        SET obj        = excluded.obj,
            updated_at = NOW();

    DELETE
    FROM schema_variants_head
    WHERE schema_variants_head.id IN (
        SELECT schema_variants_change_set_projection.id
        FROM schema_variants_change_set_projection
        WHERE schema_variants_change_set_projection.change_set_id = this_id
          AND schema_variants_change_set_projection.obj -> 'siStorable' -> 'deleted' = 'true'
    );

    INSERT INTO qualifications_head (id, obj, qualified, tenant_ids, created_at)
    SELECT qualifications_change_set_projection.id,
           qualifications_change_set_projection.obj,
           qualifications_change_set_projection.qualified,
           qualifications_change_set_projection.tenant_ids,
           qualifications_change_set_projection.created_at
    FROM qualifications_change_set_projection
    WHERE qualifications_change_set_projection.change_set_id = this_id
    ON CONFLICT(id) DO UPDATE
        SET obj        = excluded.obj,
            updated_at = NOW();

    INSERT INTO props_head (id, obj, tenant_ids, created_at)
    SELECT props_change_set_projection.id,
           props_change_set_projection.obj,
           props_change_set_projection.tenant_ids,
           props_change_set_projection.created_at
    FROM props_change_set_projection
    WHERE props_change_set_projection.change_set_id = this_id
      AND props_change_set_projection.obj -> 'siStorable' -> 'deleted' != 'true'
    ON CONFLICT(id) DO UPDATE
        SET obj        = excluded.obj,
            updated_at = NOW();

    DELETE
    FROM props_head
    WHERE props_head.id IN (
        SELECT props_change_set_projection.id
        FROM props_change_set_projection
        WHERE props_change_set_projection.change_set_id = this_id
          AND props_change_set_projection.obj -> 'siStorable' -> 'deleted' = 'true'
    );

    INSERT INTO prop_variants_head (id, obj, tenant_ids, created_at)
    SELECT prop_variants_change_set_projection.id,
           prop_variants_change_set_projection.obj,
           prop_variants_change_set_projection.tenant_ids,
           prop_variants_change_set_projection.created_at
    FROM prop_variants_change_set_projection
    WHERE prop_variants_change_set_projection.change_set_id = this_id
      AND prop_variants_change_set_projection.obj -> 'siStorable' -> 'deleted' != 'true'
    ON CONFLICT(id) DO UPDATE
        SET obj        = excluded.obj,
            updated_at = NOW();

    DELETE
    FROM prop_variants_head
    WHERE prop_variants_head.id IN (
        SELECT prop_variants_change_set_projection.id
        FROM prop_variants_change_set_projection
        WHERE prop_variants_change_set_projection.change_set_id = this_id
          AND prop_variants_change_set_projection.obj -> 'siStorable' -> 'deleted' = 'true'
    );

    INSERT INTO prop_variants_schema_variants (prop_variant_id, schema_variant_id, change_set_id, edit_session_id)
    SELECT prop_variants_schema_variants.prop_variant_id, prop_variants_schema_variants.schema_variant_id, NULL, NULL
    FROM prop_variants_schema_variants
    WHERE prop_variants_schema_variants.change_set_id = this_id
      AND edit_session_id IS NULL
      AND deleted = false;

    DELETE
    FROM prop_variants_schema_variants
    WHERE (prop_variant_id, schema_variant_id) IN (
        SELECT prop_variants_schema_variants.prop_variant_id, prop_variants_schema_variants.schema_variant_id
        FROM prop_variants_schema_variants
        WHERE prop_variants_schema_variants.change_set_id = this_id
          AND prop_variants_schema_variants.edit_session_id IS NULL
          AND deleted = true
    );

    INSERT INTO prop_variant_lineage (id, parent_prop_variant_id, child_prop_variant_id, change_set_id, edit_session_id)
    SELECT prop_variant_lineage.id,
           prop_variant_lineage.parent_prop_variant_id,
           prop_variant_lineage.child_prop_variant_id,
           NULL,
           NULL
    FROM prop_variant_lineage
    WHERE prop_variant_lineage.change_set_id = this_id
      AND edit_session_id IS NULL
      AND deleted = false;

    DELETE
    FROM prop_variant_lineage
    WHERE prop_variant_lineage.id IN (
        SELECT prop_variant_lineage.id
        FROM prop_variant_lineage
        WHERE prop_variant_lineage.change_set_id = this_id
          AND prop_variant_lineage.edit_session_id IS NULL
          AND deleted = true
    )
      AND prop_variant_lineage.change_set_id IS NULL
      AND prop_variant_lineage.edit_session_id IS NULL;
END
$$ LANGUAGE PLPGSQL VOLATILE;
