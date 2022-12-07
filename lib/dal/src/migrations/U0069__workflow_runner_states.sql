CREATE TABLE workflow_runner_states
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
    workflow_runner_id          ident                   NOT NULL,
    status                      text                     NOT NULL,
    execution_id                text,
    error_kind                  text,
    error_message               text
);

CREATE UNIQUE INDEX unique_workflow_runner_states
    ON workflow_runner_states (workflow_runner_id,
                               tenancy_universal,
                               tenancy_billing_account_ids,
                               tenancy_organization_ids,
                               tenancy_workspace_ids,
                               visibility_change_set_pk,
                               (visibility_deleted_at IS NULL))
    WHERE visibility_deleted_at IS NULL;

SELECT standard_model_table_constraints_v1('workflow_runner_states');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('workflow_runner_states', 'model', 'workflow_runner_state', 'Workflow Runner State');

CREATE OR REPLACE FUNCTION workflow_runner_state_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_workflow_runner_id ident,
    this_status text,
    this_execution_id text,
    this_error_kind text,
    this_error_message text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           workflow_runner_states%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO workflow_runner_states (tenancy_universal,
                                        tenancy_billing_account_ids,
                                        tenancy_organization_ids,
                                        tenancy_workspace_ids,
                                        visibility_change_set_pk,
                                        visibility_deleted_at,
                                        workflow_runner_id,
                                        status,
                                        execution_id,
                                        error_kind,
                                        error_message)
    VALUES (this_tenancy_record.tenancy_universal,
            this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids,
            this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at,
            this_workflow_runner_id,
            this_status,
            this_execution_id,
            this_error_kind,
            this_error_message)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
