CREATE TABLE sockets
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
    name                        text                     NOT NULL,
    edge_kind                   text                     NOT NULL,
    arity                       text                     NOT NULL,
    schematic_kind              text                     NOT NULL,
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
    this_edge_kind text,
    this_arity text,
    this_schematic_kind text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           sockets%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO sockets (tenancy_universal, tenancy_billing_account_ids, tenancy_organization_ids,
                         tenancy_workspace_ids,
                         visibility_change_set_pk, visibility_edit_session_pk, visibility_deleted_at,
                         name, edge_kind, arity, schematic_kind)
    VALUES (this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids, this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk, this_visibility_record.visibility_edit_session_pk,
            this_visibility_record.visibility_deleted_at, this_name, this_edge_kind, this_arity, this_schematic_kind)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
