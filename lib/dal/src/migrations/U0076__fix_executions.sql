CREATE TABLE fix_executions
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
    confirmation_resolver_id    bigint                   NOT NULL,
    workflow_runner_state_id    bigint                   NOT NULL,
    logs                        text[]
);

CREATE UNIQUE INDEX unique_fix_executions
    ON fix_executions (confirmation_resolver_id,
                       workflow_runner_state_id,
                       visibility_change_set_pk,
                       (visibility_deleted_at IS NULL))
    WHERE visibility_deleted_at IS NULL;

SELECT standard_model_table_constraints_v1('fix_executions');
SELECT belongs_to_table_create_v1(
               'fix_execution_belongs_to_fix_execution_batch',
               'fix_executions',
               'fix_execution_batches'
           );
INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('fix_executions', 'model', 'fix_execution', 'Fix Execution'),
       ('fix_execution_belongs_to_fix_execution_batch', 'belongs_to', 'fix_execution_batch.fix_execution',
        'Fix Execution Batch <> Fix Execution');

CREATE OR REPLACE FUNCTION fix_execution_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_confirmation_resolver_id bigint,
    this_workflow_runner_state_id bigint,
    this_logs text[],
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           fix_executions%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO fix_executions (tenancy_universal, tenancy_billing_account_ids, tenancy_organization_ids,
                                tenancy_workspace_ids, visibility_change_set_pk, visibility_deleted_at,
                                confirmation_resolver_id, workflow_runner_state_id, logs)
    VALUES (this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids, this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk, this_visibility_record.visibility_deleted_at,
            this_confirmation_resolver_id, this_workflow_runner_state_id, this_logs)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END
$$ LANGUAGE PLPGSQL VOLATILE;
