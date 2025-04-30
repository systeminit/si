CREATE TABLE job_failures
(
    pk                          ident primary key default ident_create_v1(),
    id                          ident not null default ident_create_v1(),
    tenancy_workspace_pk        ident,
    visibility_change_set_pk    ident                   NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
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

    INSERT INTO job_failures (tenancy_workspace_pk,
                              visibility_change_set_pk, 
                              kind, message)
    VALUES (this_tenancy_record.tenancy_workspace_pk,
            this_visibility_record.visibility_change_set_pk,
	    this_kind, this_message)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
