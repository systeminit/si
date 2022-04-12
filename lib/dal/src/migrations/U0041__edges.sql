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

CREATE OR REPLACE FUNCTION edge_include_component_in_system_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_component_id bigint,
    this_system_id bigint,
    this_schematic_kind text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record      tenancy_record_v1;
    this_visibility_record   visibility_record_v1;
    this_component_node_id   bigint;
    this_component_socket_id bigint;
    this_system_node_id      bigint;
    this_system_socket_id    bigint;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    SELECT
        nodes.id,
        sockets.id
    -- Using "STRICT" to ensure that this entire query must return _exactly_ one (1) row,
    -- since there should ever only be one "includes" socket for the component.
    INTO STRICT
        this_component_node_id,
        this_component_socket_id
    FROM components
            -- We're making sure the tenancy & visibility match the component's _exactly_ here, to ensure that
            -- we are getting the _exact_ join record for _this_ version of the component that we're interested in.
            INNER JOIN component_belongs_to_schema_variant
                        ON components.id = component_belongs_to_schema_variant.object_id
                            AND components.tenancy_universal = component_belongs_to_schema_variant.tenancy_universal
                            AND components.tenancy_billing_account_ids =
                                component_belongs_to_schema_variant.tenancy_billing_account_ids
                            AND components.tenancy_organization_ids =
                                component_belongs_to_schema_variant.tenancy_organization_ids
                            AND components.tenancy_workspace_ids = component_belongs_to_schema_variant.tenancy_workspace_ids
                            AND components.visibility_change_set_pk =
                                component_belongs_to_schema_variant.visibility_change_set_pk
                            AND components.visibility_edit_session_pk =
                                component_belongs_to_schema_variant.visibility_edit_session_pk
                            AND components.visibility_deleted = component_belongs_to_schema_variant.visibility_deleted
            -- We're making sure the tenancy & visibility match the component's _exactly_ here, to ensure that
            -- we are getting the _exact_ join record for _this_ version of the component that we're interested in.
            INNER JOIN node_belongs_to_component
                        ON components.id = node_belongs_to_component.belongs_to_id
                            AND components.tenancy_universal = node_belongs_to_component.tenancy_universal
                            AND components.tenancy_billing_account_ids = node_belongs_to_component.tenancy_billing_account_ids
                            AND components.tenancy_organization_ids = node_belongs_to_component.tenancy_organization_ids
                            AND components.tenancy_workspace_ids = node_belongs_to_component.tenancy_workspace_ids
                            AND components.visibility_change_set_pk = node_belongs_to_component.visibility_change_set_pk
                            AND components.visibility_edit_session_pk = node_belongs_to_component.visibility_edit_session_pk
                            AND components.visibility_deleted = node_belongs_to_component.visibility_deleted
            -- We're making sure the tenancy & visibility match the component's _exactly_ here, to ensure that
            -- we are getting the _exact_ join record for _this_ version of the component that we're interested in.
            INNER JOIN nodes
                        ON nodes.id = node_belongs_to_component.object_id
                            AND nodes.tenancy_universal = components.tenancy_universal
                            AND nodes.tenancy_billing_account_ids = components.tenancy_billing_account_ids
                            AND nodes.tenancy_organization_ids = components.tenancy_organization_ids
                            AND nodes.tenancy_workspace_ids = components.tenancy_workspace_ids
                            AND nodes.visibility_change_set_pk = node_belongs_to_component.visibility_change_set_pk
                            AND nodes.visibility_edit_session_pk = node_belongs_to_component.visibility_edit_session_pk
                            AND nodes.visibility_deleted = node_belongs_to_component.visibility_deleted
            -- We're using the in_tenancy_v1, and is_visible_v1 helpers here, because the schema_variant might not
            -- exist in the _exact_ same tenancy/visibility as the component, so we need to be able to do the
            -- fallback/lookup logic here.
            INNER JOIN schema_variants
                        ON schema_variants.id = component_belongs_to_schema_variant.belongs_to_id
                            AND in_tenancy_v1(this_tenancy, schema_variants.tenancy_universal, schema_variants.tenancy_billing_account_ids,
                                            schema_variants.tenancy_organization_ids,
                                            schema_variants.tenancy_workspace_ids)
                            AND is_visible_v1(this_visibility, schema_variants.visibility_change_set_pk, schema_variants.visibility_edit_session_pk,
                                            schema_variants.visibility_deleted)
            -- We're using the in_tenancy_v1, and is_visible_v1 helpers here, because the schema_variant might not
            -- exist in the _exact_ same tenancy/visibility as the component, so we need to be able to do the
            -- fallback/lookup logic here.
            INNER JOIN socket_many_to_many_schema_variants
                        ON schema_variants.id = socket_many_to_many_schema_variants.right_object_id
                            AND in_tenancy_v1(this_tenancy, socket_many_to_many_schema_variants.tenancy_universal,
                                            socket_many_to_many_schema_variants.tenancy_billing_account_ids,
                                            socket_many_to_many_schema_variants.tenancy_organization_ids,
                                            socket_many_to_many_schema_variants.tenancy_workspace_ids)
            -- We're using the in_tenancy_v1, and is_visible_v1 helpers here, because the socket might not
            -- exist in the _exact_ same tenancy/visibility as the component, so we need to be able to do the
            -- fallback/lookup logic here.
            INNER JOIN sockets
                        ON sockets.id = socket_many_to_many_schema_variants.left_object_id
                            AND sockets.tenancy_universal = socket_many_to_many_schema_variants.tenancy_universal
                            AND sockets.tenancy_billing_account_ids =
                                socket_many_to_many_schema_variants.tenancy_billing_account_ids
                            AND
                        sockets.tenancy_organization_ids = socket_many_to_many_schema_variants.tenancy_organization_ids
                            AND sockets.tenancy_workspace_ids = socket_many_to_many_schema_variants.tenancy_workspace_ids
                            AND sockets.edge_kind = 'includes'
                            AND sockets.schematic_kind = this_schematic_kind
    WHERE in_tenancy_v1(this_tenancy, components.tenancy_universal, components.tenancy_billing_account_ids, components.tenancy_organization_ids,
                        components.tenancy_workspace_ids)
    AND is_visible_v1(this_visibility, components.visibility_change_set_pk, components.visibility_edit_session_pk, components.visibility_deleted)
    AND components.id = this_component_id;

    SELECT
        nodes.id,
        sockets.id
    -- Using "STRICT" to ensure that this entire query must return _exactly_ one (1) row,
    -- since there should ever only be one "includes" socket for the system.
    INTO STRICT
        this_system_node_id,
        this_system_socket_id
    FROM systems
            -- We're making sure the tenancy & visibility match the system's _exactly_ here, to ensure that
            -- we are getting the _exact_ join record for _this_ version of the system that we're interested in.
            INNER JOIN system_belongs_to_schema_variant
                        ON systems.id = system_belongs_to_schema_variant.object_id
                            AND systems.tenancy_universal = system_belongs_to_schema_variant.tenancy_universal
                            AND systems.tenancy_billing_account_ids =
                                system_belongs_to_schema_variant.tenancy_billing_account_ids
                            AND systems.tenancy_organization_ids =
                                system_belongs_to_schema_variant.tenancy_organization_ids
                            AND systems.tenancy_workspace_ids = system_belongs_to_schema_variant.tenancy_workspace_ids
                            AND systems.visibility_change_set_pk =
                                system_belongs_to_schema_variant.visibility_change_set_pk
                            AND systems.visibility_edit_session_pk =
                                system_belongs_to_schema_variant.visibility_edit_session_pk
                            AND systems.visibility_deleted = system_belongs_to_schema_variant.visibility_deleted
            -- We're making sure the tenancy & visibility match the system's _exactly_ here, to ensure that
            -- we are getting the _exact_ join record for _this_ version of the system that we're interested in.
            INNER JOIN node_belongs_to_system
                        ON systems.id = node_belongs_to_system.belongs_to_id
                            AND systems.tenancy_universal = node_belongs_to_system.tenancy_universal
                            AND
                        systems.tenancy_billing_account_ids = node_belongs_to_system.tenancy_billing_account_ids
                            AND systems.tenancy_organization_ids = node_belongs_to_system.tenancy_organization_ids
                            AND systems.tenancy_workspace_ids = node_belongs_to_system.tenancy_workspace_ids
                            AND systems.visibility_change_set_pk = node_belongs_to_system.visibility_change_set_pk
                            AND systems.visibility_edit_session_pk = node_belongs_to_system.visibility_edit_session_pk
                            AND systems.visibility_deleted = node_belongs_to_system.visibility_deleted
            -- We're making sure the tenancy & visibility match the system's _exactly_ here, to ensure that
            -- we are getting the _exact_ join record for _this_ version of the system that we're interested in.
            INNER JOIN nodes
                        ON nodes.id = node_belongs_to_system.object_id
                            AND nodes.tenancy_universal = node_belongs_to_system.tenancy_universal
                            AND nodes.tenancy_billing_account_ids = node_belongs_to_system.tenancy_billing_account_ids
                            AND nodes.tenancy_organization_ids = node_belongs_to_system.tenancy_organization_ids
                            AND nodes.tenancy_workspace_ids = node_belongs_to_system.tenancy_workspace_ids
                            AND nodes.visibility_change_set_pk = node_belongs_to_system.visibility_change_set_pk
                            AND nodes.visibility_edit_session_pk = node_belongs_to_system.visibility_edit_session_pk
                            AND nodes.visibility_deleted = node_belongs_to_system.visibility_deleted
            -- We're using the in_tenancy_v1, and is_visible_v1 helpers here, because the schema_variant might not
            -- exist in the _exact_ same tenancy/visibility as the system, so we need to be able to do the
            -- fallback/lookup logic here.
            INNER JOIN schema_variants
                        ON schema_variants.id = system_belongs_to_schema_variant.belongs_to_id
                            AND in_tenancy_v1(this_tenancy, schema_variants.tenancy_universal, schema_variants.tenancy_billing_account_ids,
                                            schema_variants.tenancy_organization_ids,
                                            schema_variants.tenancy_workspace_ids)
                            AND is_visible_v1(this_visibility, schema_variants.visibility_change_set_pk, schema_variants.visibility_edit_session_pk,
                                            schema_variants.visibility_deleted)
            -- We're using the in_tenancy_v1, and is_visible_v1 helpers here, because the schema_variant might not
            -- exist in the _exact_ same tenancy/visibility as the system, so we need to be able to do the
            -- fallback/lookup logic here.
            INNER JOIN socket_many_to_many_schema_variants
                        ON schema_variants.id = socket_many_to_many_schema_variants.right_object_id
                            AND in_tenancy_v1(this_tenancy, socket_many_to_many_schema_variants.tenancy_universal,
                                            socket_many_to_many_schema_variants.tenancy_billing_account_ids,
                                            socket_many_to_many_schema_variants.tenancy_organization_ids,
                                            socket_many_to_many_schema_variants.tenancy_workspace_ids)
            -- We're using the in_tenancy_v1, and is_visible_v1 helpers here, because the socket might not
            -- exist in the _exact_ same tenancy/visibility as the system, so we need to be able to do the
            -- fallback/lookup logic here.
            INNER JOIN sockets
                        ON sockets.id = socket_many_to_many_schema_variants.left_object_id
                            AND sockets.tenancy_universal = socket_many_to_many_schema_variants.tenancy_universal
                            AND sockets.tenancy_billing_account_ids =
                                socket_many_to_many_schema_variants.tenancy_billing_account_ids
                            AND
                        sockets.tenancy_organization_ids = socket_many_to_many_schema_variants.tenancy_organization_ids
                            AND sockets.tenancy_workspace_ids = socket_many_to_many_schema_variants.tenancy_workspace_ids
                            AND sockets.edge_kind = 'output'
                            AND sockets.schematic_kind = this_schematic_kind
    WHERE in_tenancy_v1(this_tenancy, systems.tenancy_universal, systems.tenancy_billing_account_ids, systems.tenancy_organization_ids,
                        systems.tenancy_workspace_ids)
    AND is_visible_v1(this_visibility, systems.visibility_change_set_pk, systems.visibility_edit_session_pk, systems.visibility_deleted)
    AND systems.id = this_system_id;

    SELECT
        *
    INTO
        object
    FROM
        edge_create_v1(
            this_tenancy,
            this_visibility,
            'includes',
            this_component_node_id,
            'component',
            this_component_id,
            this_component_socket_id,
            this_system_node_id,
            'system',
            this_system_id,
            this_system_socket_id
        );

END;
$$ LANGUAGE plpgsql VOLATILE;

CREATE OR REPLACE FUNCTION edge_include_component_in_node_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_component_id bigint,
    this_parent_node_id bigint,
    this_schematic_kind text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record        tenancy_record_v1;
    this_visibility_record     visibility_record_v1;
    this_component_node_id     bigint;
    this_component_socket_id   bigint;
    this_parent_component_id   bigint;
    this_parent_node_socket_id bigint;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    SELECT
        nodes.id,
        sockets.id
    -- Using "STRICT" to ensure that this entire query must return _exactly_ one (1) row,
    -- since there should ever only be one "includes" socket for the component.
    INTO STRICT
        this_component_node_id,
        this_component_socket_id
    FROM components
            -- We're making sure the tenancy & visibility match the component's _exactly_ here, to ensure that
            -- we are getting the _exact_ join record for _this_ version of the component that we're interested in.
            INNER JOIN component_belongs_to_schema_variant
                        ON components.id = component_belongs_to_schema_variant.object_id
                            AND components.tenancy_universal = component_belongs_to_schema_variant.tenancy_universal
                            AND components.tenancy_billing_account_ids =
                                component_belongs_to_schema_variant.tenancy_billing_account_ids
                            AND components.tenancy_organization_ids =
                                component_belongs_to_schema_variant.tenancy_organization_ids
                            AND components.tenancy_workspace_ids = component_belongs_to_schema_variant.tenancy_workspace_ids
                            AND components.visibility_change_set_pk =
                                component_belongs_to_schema_variant.visibility_change_set_pk
                            AND components.visibility_edit_session_pk =
                                component_belongs_to_schema_variant.visibility_edit_session_pk
                            AND components.visibility_deleted = component_belongs_to_schema_variant.visibility_deleted
            -- We're making sure the tenancy & visibility match the component's _exactly_ here, to ensure that
            -- we are getting the _exact_ join record for _this_ version of the component that we're interested in.
            INNER JOIN node_belongs_to_component
                        ON components.id = node_belongs_to_component.belongs_to_id
                            AND components.tenancy_universal = node_belongs_to_component.tenancy_universal
                            AND
                        components.tenancy_billing_account_ids = node_belongs_to_component.tenancy_billing_account_ids
                            AND components.tenancy_organization_ids = node_belongs_to_component.tenancy_organization_ids
                            AND components.tenancy_workspace_ids = node_belongs_to_component.tenancy_workspace_ids
                            AND components.visibility_change_set_pk = node_belongs_to_component.visibility_change_set_pk
                            AND components.visibility_edit_session_pk = node_belongs_to_component.visibility_edit_session_pk
                            AND components.visibility_deleted = node_belongs_to_component.visibility_deleted
            -- We're making sure the tenancy & visibility match the component's _exactly_ here, to ensure that
            -- we are getting the _exact_ join record for _this_ version of the component that we're interested in.
            INNER JOIN nodes
                        ON nodes.id = node_belongs_to_component.object_id
                            AND nodes.tenancy_universal = components.tenancy_universal
                            AND nodes.tenancy_billing_account_ids = components.tenancy_billing_account_ids
                            AND nodes.tenancy_organization_ids = components.tenancy_organization_ids
                            AND nodes.tenancy_workspace_ids = components.tenancy_workspace_ids
                            AND nodes.visibility_change_set_pk = node_belongs_to_component.visibility_change_set_pk
                            AND nodes.visibility_edit_session_pk = node_belongs_to_component.visibility_edit_session_pk
                            AND nodes.visibility_deleted = node_belongs_to_component.visibility_deleted
            -- We're using the in_tenancy_v1, and is_visible_v1 helpers here, because the schema_variant might not
            -- exist in the _exact_ same tenancy/visibility as the component, so we need to be able to do the
            -- fallback/lookup logic here.
            INNER JOIN schema_variants
                        ON schema_variants.id = component_belongs_to_schema_variant.belongs_to_id
                            AND in_tenancy_v1(this_tenancy, schema_variants.tenancy_universal, schema_variants.tenancy_billing_account_ids,
                                            schema_variants.tenancy_organization_ids,
                                            schema_variants.tenancy_workspace_ids)
                            AND is_visible_v1(this_visibility, schema_variants.visibility_change_set_pk, schema_variants.visibility_edit_session_pk,
                                            schema_variants.visibility_deleted)
            -- We're using the in_tenancy_v1, and is_visible_v1 helpers here, because the schema_variant might not
            -- exist in the _exact_ same tenancy/visibility as the component, so we need to be able to do the
            -- fallback/lookup logic here.
            INNER JOIN socket_many_to_many_schema_variants
                        ON schema_variants.id = socket_many_to_many_schema_variants.right_object_id
                            AND in_tenancy_v1(this_tenancy, socket_many_to_many_schema_variants.tenancy_universal,
                                            socket_many_to_many_schema_variants.tenancy_billing_account_ids,
                                            socket_many_to_many_schema_variants.tenancy_organization_ids,
                                            socket_many_to_many_schema_variants.tenancy_workspace_ids)
            -- We're using the in_tenancy_v1, and is_visible_v1 helpers here, because the socket might not
            -- exist in the _exact_ same tenancy/visibility as the component, so we need to be able to do the
            -- fallback/lookup logic here.
            INNER JOIN sockets
                        ON sockets.id = socket_many_to_many_schema_variants.left_object_id
                            AND sockets.tenancy_universal = socket_many_to_many_schema_variants.tenancy_universal
                            AND sockets.tenancy_billing_account_ids =
                                socket_many_to_many_schema_variants.tenancy_billing_account_ids
                            AND
                        sockets.tenancy_organization_ids = socket_many_to_many_schema_variants.tenancy_organization_ids
                            AND sockets.tenancy_workspace_ids = socket_many_to_many_schema_variants.tenancy_workspace_ids
                            AND sockets.edge_kind = 'includes'
                            AND sockets.schematic_kind = this_schematic_kind
    WHERE in_tenancy_v1(this_tenancy, components.tenancy_universal, components.tenancy_billing_account_ids, components.tenancy_organization_ids,
                        components.tenancy_workspace_ids)
    AND is_visible_v1(this_visibility, components.visibility_change_set_pk, components.visibility_edit_session_pk, components.visibility_deleted)
    AND components.id = this_component_id;

    SELECT
        components.id,
        sockets.id
    -- Using "STRICT" to ensure that this entire query must return _exactly_ one (1) row,
    -- since there should ever only be one "includes" socket for the parent node.
    INTO STRICT
        this_parent_component_id,
        this_parent_node_socket_id
    FROM components
            -- We're making sure the tenancy & visibility match the component's _exactly_ here, to ensure that
            -- we are getting the _exact_ join record for _this_ version of the component that we're interested in.
            INNER JOIN component_belongs_to_schema_variant
                        ON components.id = component_belongs_to_schema_variant.object_id
                            AND components.tenancy_universal = component_belongs_to_schema_variant.tenancy_universal
                            AND components.tenancy_billing_account_ids =
                                component_belongs_to_schema_variant.tenancy_billing_account_ids
                            AND components.tenancy_organization_ids =
                                component_belongs_to_schema_variant.tenancy_organization_ids
                            AND components.tenancy_workspace_ids = component_belongs_to_schema_variant.tenancy_workspace_ids
                            AND components.visibility_change_set_pk =
                                component_belongs_to_schema_variant.visibility_change_set_pk
                            AND components.visibility_edit_session_pk =
                                component_belongs_to_schema_variant.visibility_edit_session_pk
                            AND components.visibility_deleted = component_belongs_to_schema_variant.visibility_deleted
            -- We're making sure the tenancy & visibility match the component's _exactly_ here, to ensure that
            -- we are getting the _exact_ join record for _this_ version of the component that we're interested in.
            INNER JOIN node_belongs_to_component
                        ON components.id = node_belongs_to_component.belongs_to_id
                            AND components.tenancy_universal = node_belongs_to_component.tenancy_universal
                            AND
                        components.tenancy_billing_account_ids = node_belongs_to_component.tenancy_billing_account_ids
                            AND components.tenancy_organization_ids = node_belongs_to_component.tenancy_organization_ids
                            AND components.tenancy_workspace_ids = node_belongs_to_component.tenancy_workspace_ids
                            AND components.visibility_change_set_pk = node_belongs_to_component.visibility_change_set_pk
                            AND components.visibility_edit_session_pk = node_belongs_to_component.visibility_edit_session_pk
                            AND components.visibility_deleted = node_belongs_to_component.visibility_deleted
            -- We're making sure the tenancy & visibility match the component's _exactly_ here, to ensure that
            -- we are getting the _exact_ join record for _this_ version of the component that we're interested in.
            INNER JOIN nodes
                        ON nodes.id = node_belongs_to_component.object_id
                            AND nodes.tenancy_universal = components.tenancy_universal
                            AND nodes.tenancy_billing_account_ids = components.tenancy_billing_account_ids
                            AND nodes.tenancy_organization_ids = components.tenancy_organization_ids
                            AND nodes.tenancy_workspace_ids = components.tenancy_workspace_ids
                            AND nodes.visibility_change_set_pk = node_belongs_to_component.visibility_change_set_pk
                            AND nodes.visibility_edit_session_pk = node_belongs_to_component.visibility_edit_session_pk
                            AND nodes.visibility_deleted = node_belongs_to_component.visibility_deleted
            -- We're using the in_tenancy_v1, and is_visible_v1 helpers here, because the schema_variant might not
            -- exist in the _exact_ same tenancy/visibility as the component, so we need to be able to do the
            -- fallback/lookup logic here.
            INNER JOIN schema_variants
                        ON schema_variants.id = component_belongs_to_schema_variant.belongs_to_id
                            AND in_tenancy_v1(this_tenancy, schema_variants.tenancy_universal, schema_variants.tenancy_billing_account_ids,
                                            schema_variants.tenancy_organization_ids,
                                            schema_variants.tenancy_workspace_ids)
                            AND is_visible_v1(this_visibility, schema_variants.visibility_change_set_pk, schema_variants.visibility_edit_session_pk,
                                            schema_variants.visibility_deleted)
            -- We're using the in_tenancy_v1, and is_visible_v1 helpers here, because the schema_variant might not
            -- exist in the _exact_ same tenancy/visibility as the component, so we need to be able to do the
            -- fallback/lookup logic here.
            INNER JOIN socket_many_to_many_schema_variants
                        ON schema_variants.id = socket_many_to_many_schema_variants.right_object_id
                            AND in_tenancy_v1(this_tenancy, socket_many_to_many_schema_variants.tenancy_universal,
                                            socket_many_to_many_schema_variants.tenancy_billing_account_ids,
                                            socket_many_to_many_schema_variants.tenancy_organization_ids,
                                            socket_many_to_many_schema_variants.tenancy_workspace_ids)
            -- We're using the in_tenancy_v1, and is_visible_v1 helpers here, because the socket might not
            -- exist in the _exact_ same tenancy/visibility as the component, so we need to be able to do the
            -- fallback/lookup logic here.
            INNER JOIN sockets
                        ON sockets.id = socket_many_to_many_schema_variants.left_object_id
                            AND sockets.tenancy_universal = socket_many_to_many_schema_variants.tenancy_universal
                            AND sockets.tenancy_billing_account_ids =
                                socket_many_to_many_schema_variants.tenancy_billing_account_ids
                            AND
                        sockets.tenancy_organization_ids = socket_many_to_many_schema_variants.tenancy_organization_ids
                            AND sockets.tenancy_workspace_ids = socket_many_to_many_schema_variants.tenancy_workspace_ids
                            AND sockets.edge_kind = 'output'
                            AND sockets.schematic_kind = this_schematic_kind
    WHERE in_tenancy_v1(this_tenancy, components.tenancy_universal, components.tenancy_billing_account_ids, components.tenancy_organization_ids,
                        components.tenancy_workspace_ids)
    AND is_visible_v1(this_visibility, components.visibility_change_set_pk, components.visibility_edit_session_pk, components.visibility_deleted)
    AND nodes.id = this_parent_node_id;

    SELECT
        *
    INTO
        object
    FROM
        edge_create_v1(
            this_tenancy,
            this_visibility,
            'includes',
            this_component_node_id,
            'component',
            this_component_id,
            this_component_socket_id,
            this_parent_node_id,
            'component',
            this_parent_component_id,
            this_parent_node_socket_id
        );

END;
$$ LANGUAGE plpgsql VOLATILE;
