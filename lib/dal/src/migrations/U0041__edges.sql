CREATE TABLE edges
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
    kind                        text                     NOT NULL,
    head_node_id                bigint                   NOT NULL,
    head_object_kind            text                     NOT NULL,
    head_object_id              bigint                   NOT NULL,
    head_socket_id              bigint                   NOT NULL,
    tail_node_id                bigint                   NOT NULL,
    tail_object_kind            text                     NOT NULL,
    tail_object_id              bigint                   NOT NULL,
    tail_socket_id              bigint                   NOT NULL
);
SELECT standard_model_table_constraints_v1('edges');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('edges', 'model', 'edge', 'Edge');

CREATE OR REPLACE FUNCTION edge_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_kind text,
    this_head_node_id bigint,
    this_head_object_kind text,
    this_head_object_id bigint,
    this_head_socket_id bigint,
    this_tail_node_id bigint,
    this_tail_object_kind text,
    this_tail_object_id bigint,
    this_tail_socket_id bigint,
        OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           edges%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO edges (tenancy_universal, tenancy_billing_account_ids, tenancy_organization_ids,
                       tenancy_workspace_ids,
                       visibility_change_set_pk, visibility_edit_session_pk, visibility_deleted, kind,
                       head_node_id, head_object_kind, head_object_id, head_socket_id,
                       tail_node_id, tail_object_kind, tail_object_id, tail_socket_id)
    VALUES (this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids, this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk, this_visibility_record.visibility_edit_session_pk,
            this_visibility_record.visibility_deleted, this_kind, this_head_node_id, this_head_object_kind,
            this_head_object_id, this_head_socket_id, this_tail_node_id, this_tail_object_kind, this_tail_object_id,
            this_tail_socket_id)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;