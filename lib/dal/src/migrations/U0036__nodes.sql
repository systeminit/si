CREATE TABLE nodes
(
    pk                       ident primary key                 default ident_create_v1(),
    id                       ident                    not null default ident_create_v1(),
    tenancy_workspace_pk     ident,
    visibility_change_set_pk ident                    NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at    timestamp with time zone,
    created_at               timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at               timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    kind                     text                     NOT NULL,
    x                        text                     NOT NULL DEFAULT '0',
    y                        text                     NOT NULL DEFAULT '0',
    width                    text,
    height                   text
);
SELECT standard_model_table_constraints_v1('nodes');
SELECT belongs_to_table_create_v1('node_belongs_to_component', 'nodes', 'components');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('nodes', 'model', 'node', 'Node'),
       ('node_belongs_to_component', 'belongs_to', 'node.component', 'Node <> Component');

CREATE OR REPLACE FUNCTION node_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_kind text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           nodes%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO nodes (tenancy_workspace_pk,
                       visibility_change_set_pk, kind)
    VALUES (this_tenancy_record.tenancy_workspace_pk,
            this_visibility_record.visibility_change_set_pk, this_kind)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
