CREATE TABLE node_positions
(
    pk                          bigserial PRIMARY KEY,
    id                          bigserial                NOT NULL,
    tenancy_universal           bool                     NOT NULL,
    tenancy_billing_account_ids bigint[],
    tenancy_organization_ids    bigint[],
    tenancy_workspace_ids       bigint[],
    visibility_change_set_pk    bigint                   NOT NULL DEFAULT -1,
    visibility_edit_session_pk  bigint                   NOT NULL DEFAULT -1,
    visibility_deleted          bool,
    created_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    schematic_kind              text                     NOT NULL,
    root_node_id                bigint                   NOT NULL,
    system_id                   bigint,
    x                           text                     NOT NULL,
    y                           text                     NOT NULL
);
SELECT standard_model_table_constraints_v1('node_positions');
SELECT belongs_to_table_create_v1('node_position_belongs_to_node', 'node_positions', 'nodes');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('node_positions', 'model', 'node_position', 'Node Position'),
        ('node_position_belongs_to_node', 'belongs_to', 'node_position.node', 'Node Position <> Node');

CREATE OR REPLACE FUNCTION node_position_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_schematic_kind text,
    this_root_node_id bigint,
    this_x text,
    this_y text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           node_positions%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO node_positions (tenancy_universal, tenancy_billing_account_ids, tenancy_organization_ids,
                         tenancy_workspace_ids,
                         visibility_change_set_pk, visibility_edit_session_pk, visibility_deleted,
                         schematic_kind, root_node_id, x, y)
    VALUES (this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids, this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk, this_visibility_record.visibility_edit_session_pk,
            this_visibility_record.visibility_deleted, this_schematic_kind, this_root_node_id, this_x, this_y)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;