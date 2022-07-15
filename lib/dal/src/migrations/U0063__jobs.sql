CREATE TABLE job_failures
(
    pk                          bigserial PRIMARY KEY,
    id                          bigserial                NOT NULL,
    tenancy_universal           bool                     NOT NULL,
    tenancy_billing_account_ids bigint[],
    tenancy_organization_ids    bigint[],
    tenancy_workspace_ids       bigint[],
    visibility_change_set_pk    bigint                   NOT NULL DEFAULT -1,
    visibility_edit_session_pk  bigint                   NOT NULL DEFAULT -1,
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    kind                        text                     NOT NULL,
    message                     text                     NOT NULL,
    solved                      bool                     NOT NULL DEFAULT false
);
SELECT standard_model_table_constraints_v1('job_failures');

CREATE OR REPLACE FUNCTION job_failure_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_kind text,
    this_message text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           job_failures%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO job_failures (tenancy_universal, tenancy_billing_account_ids, tenancy_organization_ids,
                       tenancy_workspace_ids,
                       visibility_change_set_pk, visibility_edit_session_pk, visibility_deleted_at,
                       kind, message)
    VALUES (this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids, this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk, this_visibility_record.visibility_edit_session_pk,
            this_visibility_record.visibility_deleted_at, this_kind, this_message)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
