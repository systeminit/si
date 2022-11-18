CREATE TABLE node_positions
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
    diagram_kind                text                     NOT NULL,
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
    this_diagram_kind text,
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
                                visibility_change_set_pk, visibility_deleted_at,
                                diagram_kind, x, y)
    VALUES (this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids, this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at, this_diagram_kind,
            this_x, this_y)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
