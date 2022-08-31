CREATE TABLE workflow_runners
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
    workflow_prototype_id       bigint                   NOT NULL,
    workflow_resolver_id        bigint                   NOT NULL,
    func_id                     bigint                   NOT NULL,
    func_binding_id             bigint                   NOT NULL,
    component_id                bigint                   NOT NULL,
    schema_id                   bigint                   NOT NULL,
    schema_variant_id           bigint                   NOT NULL,
    system_id                   bigint                   NOT NULL
);
SELECT standard_model_table_constraints_v1('workflow_runners');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('workflow_runners', 'model', 'workflow_runner', 'workflow runner');

CREATE OR REPLACE FUNCTION workflow_runner_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_workflow_prototype_id bigint,
    this_workflow_resolver_id bigint,
    this_func_id bigint,
    this_func_binding_id bigint,
    this_component_id bigint,
    this_schema_id bigint,
    this_schema_variant_id bigint,
    this_system_id bigint,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           workflow_runners%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO workflow_runners (tenancy_universal,
                                         tenancy_billing_account_ids,
                                         tenancy_organization_ids,
                                         tenancy_workspace_ids,
                                         visibility_change_set_pk,
                                         visibility_deleted_at,
                                         workflow_prototype_id,
                                         workflow_resolver_id,
                                         func_id,
                                         func_binding_id,
                                         component_id,
                                         schema_id,
                                         schema_variant_id,
                                         system_id)
    VALUES (this_tenancy_record.tenancy_universal,
            this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids,
            this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at,
            this_workflow_prototype_id,
            this_workflow_resolver_id,
            this_func_id,
            this_func_binding_id,
            this_component_id,
            this_schema_id,
            this_schema_variant_id,
            this_system_id)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
