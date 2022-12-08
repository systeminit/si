CREATE TABLE qualification_prototypes
(
    pk                          ident primary key default ident_create_v1(),
    id                          ident not null default ident_create_v1(),
    tenancy_universal           bool                     NOT NULL,
    tenancy_billing_account_ids ident[],
    tenancy_organization_ids    ident[],
    tenancy_workspace_ids       ident[],
    visibility_change_set_pk    ident                   NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    func_id                     ident                   NOT NULL,
    component_id                ident                   NOT NULL,
    schema_id                   ident                   NOT NULL,
    schema_variant_id           ident                   NOT NULL
);
SELECT standard_model_table_constraints_v1('qualification_prototypes');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('qualification_prototypes', 'model', 'qualification_prototype', 'Qualification Prototype');

CREATE OR REPLACE FUNCTION qualification_prototype_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_func_id ident,
    this_component_id ident,
    this_schema_id ident,
    this_schema_variant_id ident,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           qualification_prototypes%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO qualification_prototypes (tenancy_universal,
                                          tenancy_billing_account_ids,
                                          tenancy_organization_ids,
                                          tenancy_workspace_ids,
                                          visibility_change_set_pk,
                                          visibility_deleted_at,
                                          func_id,
                                          component_id,
                                          schema_id,
                                          schema_variant_id)
    VALUES (this_tenancy_record.tenancy_universal,
            this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids,
            this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at,
            this_func_id,
            this_component_id,
            this_schema_id,
            this_schema_variant_id)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
