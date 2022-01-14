CREATE TABLE attribute_resolvers
(
    pk                          bigserial PRIMARY KEY,
    id                          bigserial                NOT NULL,
    tenancy_universal           bool                     NOT NULL,
    tenancy_billing_account_ids bigint[],
    tenancy_organization_ids    bigint[],
    tenancy_workspace_ids       bigint[],
    visibility_change_set_pk    bigint                   NOT NULL DEFAULT -1,
    visibility_edit_session_pk  bigint                   NOT NULL DEFAULT -1,
    visibility_deleted          bool,
    created_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    func_id                     bigint                   NOT NULL,
    func_binding_id             bigint                   NOT NULL,
    prop_id                     bigint,
    component_id                bigint,
    schema_id                   bigint,
    schema_variant_id           bigint,
    system_id                   bigint
);
SELECT standard_model_table_constraints_v1('attribute_resolvers');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('attribute_resolvers', 'model', 'attribute_resolver', 'Attribute Resolver');

CREATE OR REPLACE FUNCTION attribute_resolver_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_func_id bigint,
    this_func_binding_id bigint,
    this_prop_id bigint,
    this_component_id bigint,
    this_schema_id bigint,
    this_schema_variant_id bigint,
    this_system_id bigint,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           attribute_resolvers%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO attribute_resolvers (tenancy_universal,
                                     tenancy_billing_account_ids,
                                     tenancy_organization_ids,
                                     tenancy_workspace_ids,
                                     visibility_change_set_pk,
                                     visibility_edit_session_pk,
                                     visibility_deleted,
                                     func_id,
                                     func_binding_id,
                                     prop_id,
                                     component_id,
                                     schema_id,
                                     schema_variant_id,
                                     system_id)
    VALUES (this_tenancy_record.tenancy_universal,
            this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids,
            this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_edit_session_pk,
            this_visibility_record.visibility_deleted,
            this_func_id,
            this_func_binding_id,
            this_prop_id,
            this_component_id,
            this_schema_id,
            this_schema_variant_id,
            this_system_id)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION attribute_resolver_upsert_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_func_id bigint,
    this_func_binding_id bigint,
    this_prop_id bigint,
    this_component_id bigint,
    this_schema_id bigint,
    this_schema_variant_id bigint,
    this_system_id bigint,
    OUT object json, OUT created boolean) AS
$$
DECLARE
    this_tenancy_record        tenancy_record_v1;
    this_visibility_record     visibility_record_v1;
    this_update_id             bigint;
    this_change_set_visibility jsonb;
    this_head_visibility       jsonb;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);
    created := false;
    this_change_set_visibility := jsonb_build_object(
            'visibility_change_set_pk',
            this_visibility_record.visibility_change_set_pk,
            'visibility_edit_session_pk',
            -1,
            'visibility_deleted',
            this_visibility_record.visibility_deleted);
    this_head_visibility := jsonb_build_object(
            'visibility_change_set_pk',
            -1,
            'visibility_edit_session_pk',
            -1,
            'visibility_deleted',
            this_visibility_record.visibility_deleted);

    -- If we have an ID at all in this query, its because something in our
    -- possible visibilities has an AttributeResolver with our criteria!
    SELECT id
    FROM attribute_resolvers
    WHERE in_tenancy_v1(
            this_tenancy,
            attribute_resolvers.tenancy_universal,
            attribute_resolvers.tenancy_billing_account_ids,
            attribute_resolvers.tenancy_organization_ids,
            attribute_resolvers.tenancy_workspace_ids)
      AND (
            is_visible_v1(
                    this_visibility,
                    attribute_resolvers.visibility_change_set_pk,
                    attribute_resolvers.visibility_edit_session_pk,
                    attribute_resolvers.visibility_deleted)
            OR
            is_visible_v1(
                    this_change_set_visibility,
                    attribute_resolvers.visibility_change_set_pk,
                    attribute_resolvers.visibility_edit_session_pk,
                    attribute_resolvers.visibility_deleted)
            OR
            is_visible_v1(
                    this_visibility,
                    attribute_resolvers.visibility_change_set_pk,
                    attribute_resolvers.visibility_edit_session_pk,
                    attribute_resolvers.visibility_deleted)
        )
      AND attribute_resolvers.prop_id = this_prop_id
      AND attribute_resolvers.component_id = this_component_id
      AND attribute_resolvers.schema_id = this_schema_id
      AND attribute_resolvers.schema_variant_id = this_schema_variant_id
      AND attribute_resolvers.system_id = this_system_id
    LIMIT 1
    INTO this_update_id;

    IF this_update_id IS NULL THEN
        created := true;
        SELECT *
        FROM attribute_resolver_create_v1(
                this_tenancy,
                this_visibility,
                this_func_id,
                this_func_binding_id,
                this_prop_id,
                this_component_id,
                this_schema_id,
                this_schema_variant_id,
                this_system_id
            )
        INTO object;
    ELSE
        PERFORM update_by_id_v1('attribute_resolvers', 'func_id', this_tenancy, this_visibility, this_update_id,
                                this_func_id);
        PERFORM update_by_id_v1('attribute_resolvers', 'func_binding_id', this_tenancy, this_visibility, this_update_id,
                                this_func_binding_id);
        SELECT row_to_json(attribute_resolvers.*)
        FROM attribute_resolvers
        WHERE in_tenancy_v1(
                this_tenancy,
                attribute_resolvers.tenancy_universal,
                attribute_resolvers.tenancy_billing_account_ids,
                attribute_resolvers.tenancy_organization_ids,
                attribute_resolvers.tenancy_workspace_ids)
          AND is_visible_v1(
                this_visibility,
                attribute_resolvers.visibility_change_set_pk,
                attribute_resolvers.visibility_edit_session_pk,
                attribute_resolvers.visibility_deleted)
          AND attribute_resolvers.prop_id = this_prop_id
          AND attribute_resolvers.component_id = this_component_id
          AND attribute_resolvers.schema_id = this_schema_id
          AND attribute_resolvers.schema_variant_id = this_schema_variant_id
          AND attribute_resolvers.system_id = this_system_id
        INTO object;
    END IF;
END;
$$ LANGUAGE PLPGSQL VOLATILE;


