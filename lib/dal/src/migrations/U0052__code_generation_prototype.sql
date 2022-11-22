CREATE TABLE code_generation_prototypes
(
    pk                          bigserial PRIMARY KEY,
    id                          bigserial                NOT NULL,
    tenancy_universal           bool                     NOT NULL,
    tenancy_billing_account_ids bigint[],
    tenancy_organization_ids    bigint[],
    tenancy_workspace_ids       bigint[],
    visibility_change_set_pk    bigint                   NOT NULL DEFAULT -1,
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    func_id                     bigint                   NOT NULL,
    args                        jsonb                    NOT NULL,
    tree_prop_id                bigint                   NOT NULL,
    code_prop_id                bigint                   NOT NULL,
    format_prop_id              bigint                   NOT NULL,
    schema_variant_id           bigint                   NOT NULL
);

CREATE UNIQUE INDEX unique_code_generation_prototypes_for_schema_variants
    ON code_generation_prototypes (func_id,
                                   schema_variant_id,
                                   visibility_change_set_pk,
                                   (visibility_deleted_at IS NULL))
    WHERE visibility_deleted_at IS NULL;

SELECT standard_model_table_constraints_v1('code_generation_prototypes');
INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('code_generation_prototypes', 'model', 'code_generation_prototype', 'Code Generation Prototype');

CREATE OR REPLACE FUNCTION code_generation_prototype_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_func_id bigint,
    this_args jsonb,
    this_tree_prop_id bigint,
    this_code_prop_id bigint,
    this_format_prop_id bigint,
    this_schema_variant_id bigint,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           code_generation_prototypes%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO code_generation_prototypes (tenancy_universal,
                                            tenancy_billing_account_ids,
                                            tenancy_organization_ids,
                                            tenancy_workspace_ids,
                                            visibility_change_set_pk,
                                            visibility_deleted_at,
                                            func_id,
                                            args,
                                            tree_prop_id,
                                            code_prop_id,
                                            format_prop_id,
                                            schema_variant_id)
    VALUES (this_tenancy_record.tenancy_universal,
            this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids,
            this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at,
            this_func_id,
            this_args,
            this_tree_prop_id,
            this_code_prop_id,
            this_format_prop_id,
            this_schema_variant_id)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
