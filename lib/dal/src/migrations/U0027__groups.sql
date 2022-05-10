CREATE TABLE groups
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
    name                        text                     NOT NULL
);
SELECT standard_model_table_constraints_v1('groups');
SELECT belongs_to_table_create_v1('group_belongs_to_billing_account', 'groups', 'billing_accounts');
SELECT many_to_many_table_create_v1('group_many_to_many_users', 'groups', 'users');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('groups', 'model', 'group', 'Group'),
       ('group_belongs_to_billing_account', 'belongs_to', 'group.billing_account', 'Group <> Billing Account'),
       ('group_many_to_many_users', 'many_to_many', 'group.users', 'Group <> User');

CREATE OR REPLACE FUNCTION group_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_name text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           groups%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO groups (tenancy_universal, tenancy_billing_account_ids, tenancy_organization_ids, tenancy_workspace_ids,
                        visibility_change_set_pk, visibility_edit_session_pk, visibility_deleted_at, name)
    VALUES (this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids, this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk, this_visibility_record.visibility_edit_session_pk,
            this_visibility_record.visibility_deleted_at, this_name)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
