CREATE TABLE summary_qualifications
(
    pk                          ident primary key default ident_create_v1(),
    id                          ident not null default ident_create_v1(),
    component_id                ident NOT NULL,
    component_name              text NOT NULL,
    tenancy_workspace_pk        ident,
    visibility_change_set_pk    ident NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    total                       bigint,
    warned                      bigint,
    succeeded                   bigint,
    failed                      bigint
);

SELECT standard_model_table_constraints_v1('summary_qualifications');
INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('summary_qualifications', 'model', 'summary_qualifications', 'Summary Qualifications');

CREATE OR REPLACE FUNCTION summary_qualification_update_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_component_id ident,
    this_component_name text,
    this_total bigint,
    this_warned bigint,
    this_succeeded bigint,
    this_failed bigint,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           summary_qualifications%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO summary_qualifications
        (id, component_id, component_name, tenancy_workspace_pk, visibility_change_set_pk, total, warned, succeeded, failed)
    VALUES
        (this_component_id, this_component_id, this_component_name, this_tenancy_record.tenancy_workspace_pk, this_visibility_record.visibility_change_set_pk,
         this_total, this_warned, this_succeeded, this_failed)
    ON CONFLICT (id, tenancy_workspace_pk, visibility_change_set_pk)
    DO UPDATE SET component_name = this_component_name, total = this_total, warned = this_warned, succeeded = this_succeeded, failed = this_failed
    RETURNING * INTO this_new_row;
END
$$ LANGUAGE PLPGSQL VOLATILE;
