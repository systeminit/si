CREATE TABLE node_positions
(
    pk                          ident primary key default ident_create_v1(),
    id                          ident not null default ident_create_v1(),
    tenancy_workspace_pks       ident[],
    visibility_change_set_pk    ident                   NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    diagram_kind                text                     NOT NULL,
    x                           text                     NOT NULL,
    y                           text                     NOT NULL,
    width                       text,
    height                      text
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
    this_width text,
    this_height text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           node_positions%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO node_positions (tenancy_workspace_pks,
                                visibility_change_set_pk, visibility_deleted_at,
                                diagram_kind, x, y, width, height)
    VALUES (this_tenancy_record.tenancy_workspace_pks,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at, this_diagram_kind,
            this_x, this_y, this_width, this_height)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
