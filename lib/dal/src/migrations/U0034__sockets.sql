CREATE TABLE sockets
(
    pk                          ident primary key                 default ident_create_v1(),
    id                          ident                    not null default ident_create_v1(),
    tenancy_billing_account_pks ident[],
    tenancy_organization_pks    ident[],
    tenancy_workspace_pks       ident[],
    visibility_change_set_pk    ident                    NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    name                        text                     NOT NULL,
    kind                        text                     NOT NULL,
    edge_kind                   text                     NOT NULL,
    arity                       text                     NOT NULL,
    diagram_kind                text                     NOT NULL,
    required                    bool                     NOT NULL DEFAULT false
);
SELECT standard_model_table_constraints_v1('sockets');
SELECT many_to_many_table_create_v1('socket_many_to_many_schema_variants', 'sockets', 'schema_variants');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('sockets', 'model', 'socket', 'Socket'),
       ('socket_many_to_many_schema_variants', 'many_to_many', 'socket.types', 'Socket <> Schema Variant');

CREATE OR REPLACE FUNCTION socket_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_name text,
    this_kind text,
    this_edge_kind text,
    this_arity text,
    this_diagram_kind text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           sockets%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO sockets (tenancy_billing_account_pks, tenancy_organization_pks,
                         tenancy_workspace_pks,
                         visibility_change_set_pk, visibility_deleted_at,
                         name, kind, edge_kind, arity, diagram_kind)
    VALUES (this_tenancy_record.tenancy_billing_account_pks,
            this_tenancy_record.tenancy_organization_pks, this_tenancy_record.tenancy_workspace_pks,
            this_visibility_record.visibility_change_set_pk, this_visibility_record.visibility_deleted_at,
            this_name, this_kind, this_edge_kind, this_arity, this_diagram_kind)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
