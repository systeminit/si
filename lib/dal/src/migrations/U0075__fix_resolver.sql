CREATE TABLE fix_resolvers
(
    pk                          ident primary key default ident_create_v1(),
    id                          ident not null default ident_create_v1(),
    tenancy_universal           bool                     NOT NULL,
    tenancy_billing_account_ids ident[],
    tenancy_organization_ids    ident[],
    tenancy_workspace_ids       ident[],
    visibility_change_set_pk    ident                   NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    workflow_prototype_id       ident                   NOT NULL,
    confirmation_resolver_id    ident                   NOT NULL,
    success                     bool,
    component_id                ident                   NOT NULL,
    schema_id                   ident                   NOT NULL,
    schema_variant_id           ident                   NOT NULL
);
SELECT standard_model_table_constraints_v1('fix_resolvers');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('fix_resolvers', 'model', 'fix_resolver', 'Fix Resolver');

CREATE OR REPLACE FUNCTION fix_resolver_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_workflow_prototype_id ident,
    this_confirmation_resolver_id ident,
    this_success bool,
    this_component_id ident,
    this_schema_id ident,
    this_schema_variant_id ident,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           fix_resolvers%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO fix_resolvers (tenancy_universal,
                                         tenancy_billing_account_ids,
                                         tenancy_organization_ids,
                                         tenancy_workspace_ids,
                                         visibility_change_set_pk,
                                         visibility_deleted_at,
                                         workflow_prototype_id,
                                         confirmation_resolver_id,
                                         success,
                                         component_id,
                                         schema_id,
                                         schema_variant_id)
    VALUES (this_tenancy_record.tenancy_universal,
            this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids,
            this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at,
            this_workflow_prototype_id,
            this_confirmation_resolver_id,
            this_success,
            this_component_id,
            this_schema_id,
            this_schema_variant_id)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
