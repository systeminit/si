CREATE TABLE capabilities
(
    pk                          ident primary key default ident_create_v1(),
    id                          ident not null default ident_create_v1(),
    tenancy_workspace_pk        ident,
    visibility_change_set_pk    ident                   NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    subject                     text                     NOT NULL,
    action                      text                     NOT NULL
);
SELECT standard_model_table_constraints_v1('capabilities');
SELECT belongs_to_table_create_v1('capability_belongs_to_group', 'capabilities', 'groups');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('capabilities', 'model', 'capability', 'Capability'),
       ('capability_belongs_to_group', 'belongs_to', 'capability.group', 'Capability <> Group');

CREATE OR REPLACE FUNCTION capability_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_subject text,
    this_action text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           capabilities%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO capabilities (tenancy_workspace_pk, visibility_change_set_pk, visibility_deleted_at, subject, action)
    VALUES (this_tenancy_record.tenancy_workspace_pk,
            this_visibility_record.visibility_change_set_pk, this_visibility_record.visibility_deleted_at,
            this_subject, this_action)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
