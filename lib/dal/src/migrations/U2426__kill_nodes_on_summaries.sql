-- Component Summaries
ALTER TABLE summary_diagram_components
    DROP node_id,
    DROP child_node_ids;
ALTER TABLE summary_diagram_components
    RENAME parent_node_id TO parent_component_id;

CREATE OR REPLACE PROCEDURE force_component_summary_to_changeset_v3(
    this_tenancy_record tenancy_record_v1,
    this_visibility_record visibility_record_v1,
    this_component_id ident
)
AS
$$
BEGIN
    -- check to see if there is a row already for this change set. If there isn't, we copy the HEAD
    -- row with a few changes.
    IF NOT EXISTS (SELECT
                   FROM summary_diagram_components
                   WHERE component_id = this_component_id
                     AND tenancy_workspace_pk = this_tenancy_record.tenancy_workspace_pk
                     AND visibility_change_set_pk = this_visibility_record.visibility_change_set_pk) THEN

        INSERT INTO summary_diagram_components (id, tenancy_workspace_pk, visibility_change_set_pk,
                                                visibility_deleted_at, created_at, component_id,
                                                display_name, schema_name,
                                                schema_id, schema_variant_id,
                                                schema_variant_name, schema_category, position, size, color, node_type,
                                                change_status, has_resource, created_info, updated_info, deleted_info,
                                                sockets, parent_component_id)
        SELECT id,
               tenancy_workspace_pk,
               this_visibility_record.visibility_change_set_pk AS visibility_change_set_pk,
               this_visibility_record.visibility_deleted_at    AS visibility_deleted_at,
               created_at,
               component_id,
               display_name,
               schema_name,
               schema_id,
               schema_variant_id,
               schema_variant_name,
               schema_category,
               position,
               size,
               color,
               node_type,
               'added',
               has_resource,
               created_info,
               updated_info,
               deleted_info,
               sockets,
               parent_component_id
        FROM summary_diagram_components
        WHERE id = this_component_id
          AND tenancy_workspace_pk = this_tenancy_record.tenancy_workspace_pk
          AND visibility_change_set_pk = ident_nil_v1();
    END IF;
END
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION summary_diagram_component_create_v2(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_id ident,
    this_schema_name text,
    this_schema_id ident,
    this_schema_variant_id ident,
    this_schema_variant_name text,
    this_schema_category text,
    this_sockets jsonb,
    this_display_name text,
    this_position jsonb,
    this_size jsonb,
    this_color text,
    this_node_type text,
    this_change_status text,
    this_has_resource boolean,
    this_created_info jsonb,
    this_updated_info jsonb,
    this_deleted_info jsonb,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           summary_diagram_components%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO summary_diagram_components (id, tenancy_workspace_pk, visibility_change_set_pk, visibility_deleted_at,
                                            component_id, display_name, schema_name,
                                            schema_id, schema_variant_id, schema_variant_name, schema_category,
                                            position, size, color, node_type, change_status, has_resource, created_info,
                                            updated_info, deleted_info, sockets, child_node_ids)
    VALUES (this_id, this_tenancy_record.tenancy_workspace_pk, this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at,
            this_id, this_display_name, this_schema_name, this_schema_id, this_schema_variant_id,
            this_schema_variant_name,
            this_schema_category, this_position, this_size, this_color, this_node_type, this_change_status,
            this_has_resource, this_created_info, this_updated_info, this_deleted_info,
            this_sockets, jsonb_build_array())
    RETURNING * INTO this_new_row;
END
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION summary_diagram_component_update_geometry_v3(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_component_id ident,
    this_position jsonb,
    this_size jsonb,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           summary_diagram_components%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    CALL force_component_summary_to_changeset_v3(
            this_tenancy_record,
            this_visibility_record,
            this_component_id
         );

    UPDATE summary_diagram_components
    SET position = this_position,
        size     = this_size
    WHERE component_id = this_component_id
      AND tenancy_workspace_pk = this_tenancy_record.tenancy_workspace_pk
      AND visibility_change_set_pk = this_visibility_record.visibility_change_set_pk
    RETURNING * INTO this_new_row;
END
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION summary_diagram_component_update_v3(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_component_id ident,
    this_name text,
    this_color text,
    this_component_type text,
    this_has_resource bool,
    this_updated_info jsonb,
    this_deleted_at timestamp with time zone,
    this_deleted_info jsonb,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           summary_diagram_components%ROWTYPE;
    this_change_status     text;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    CALL force_component_summary_to_changeset_v3(
            this_tenancy_record,
            this_visibility_record,
            this_component_id
         );


    IF this_deleted_at IS NOT NULL THEN
        this_change_status := 'deleted';
    ELSIF NOT component_summary_exists_in_head_v1(
            this_tenancy_record,
            this_component_id
              )
    THEN
        this_change_status := 'added';
    ELSE
        this_change_status := 'modified';
    END IF;

    UPDATE summary_diagram_components
    SET display_name=this_name,
        color=this_color,
        node_type=this_component_type,
        has_resource=this_has_resource,
        updated_info=this_updated_info,
        visibility_deleted_at = this_deleted_at,
        deleted_info=this_deleted_info,
        change_status=this_change_status
    WHERE component_id = this_component_id
      AND tenancy_workspace_pk = this_tenancy_record.tenancy_workspace_pk
      AND visibility_change_set_pk = this_visibility_record.visibility_change_set_pk
    RETURNING * INTO this_new_row;
END
$$ LANGUAGE PLPGSQL VOLATILE;

-- replaces summary_diagram_component_set_parent_node_id_v3
CREATE OR REPLACE FUNCTION summary_diagram_component_set_parent_id_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_component_id ident,
    this_parent_component_id ident,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           summary_diagram_components%ROWTYPE;
    this_change_status     text;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    CALL force_component_summary_to_changeset_v3(
            this_tenancy_record,
            this_visibility_record,
            this_component_id
         );


    IF NOT component_summary_exists_in_head_v1(
            this_tenancy_record,
            this_component_id
           )
    THEN
        this_change_status := 'added';
    ELSE
        this_change_status := 'modified';
    END IF;

    UPDATE summary_diagram_components
    SET parent_component_id=this_parent_component_id,
        change_status=this_change_status
    WHERE component_id = this_component_id
      AND tenancy_workspace_pk = this_tenancy_record.tenancy_workspace_pk
      AND visibility_change_set_pk = this_visibility_record.visibility_change_set_pk
    RETURNING * INTO this_new_row;
END
$$ LANGUAGE PLPGSQL VOLATILE;

-- Edge Summaries
ALTER TABLE summary_diagram_edges
    RENAME from_node_id TO from_component_id;
ALTER TABLE summary_diagram_edges
    RENAME to_node_id TO to_component_id;

CREATE OR REPLACE FUNCTION summary_diagram_edge_create_v2(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_id ident,
    this_from_component_id ident,
    this_from_socket_id ident,
    this_to_component_id ident,
    this_to_socket_id ident,
    this_created_info jsonb,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           summary_diagram_edges%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO summary_diagram_edges (id, tenancy_workspace_pk, visibility_change_set_pk, visibility_deleted_at,
                                       edge_id, from_component_id, from_socket_id, to_component_id,
                                       to_socket_id, change_status, created_info)
    VALUES (this_id, this_tenancy_record.tenancy_workspace_pk, this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at, this_id, this_from_component_id, this_from_socket_id,
            this_to_component_id, this_to_socket_id, 'added', this_created_info)
    RETURNING * INTO this_new_row;
END
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION summary_diagram_edge_delete_v2(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_id ident,
    this_visibility_deleted_at timestamp with time zone,
    this_deleted_info jsonb,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           summary_diagram_edges%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    -- First, we check to see if there is a row already for this change set. If there isn't, we copy the HEAD
    -- row with a few changes.
    IF NOT EXISTS (SELECT
                   FROM summary_diagram_edges
                   WHERE id = this_id
                     AND tenancy_workspace_pk = this_tenancy_record.tenancy_workspace_pk
                     AND visibility_change_set_pk = this_visibility_record.visibility_change_set_pk) THEN
        INSERT INTO summary_diagram_edges
        (id, tenancy_workspace_pk, visibility_change_set_pk, visibility_deleted_at, created_at, updated_at, edge_id,
         from_component_id, from_socket_id, to_component_id, to_socket_id, change_status, created_info, deleted_info)
        SELECT id,
               tenancy_workspace_pk,
               this_visibility_record.visibility_change_set_pk AS visibility_change_set_pk,
               this_visibility_deleted_at,
               created_at,
               updated_at,
               edge_id,
               from_component_id,
               from_socket_id,
               to_component_id,
               to_socket_id,
               change_status,
               created_info,
               deleted_info
        FROM summary_diagram_edges
        WHERE id = this_id
          AND tenancy_workspace_pk = this_tenancy_record.tenancy_workspace_pk
          AND visibility_change_set_pk = ident_nil_v1();
    END IF;

    UPDATE summary_diagram_edges
    SET visibility_deleted_at = this_visibility_deleted_at,
        deleted_info          = this_deleted_info,
        change_status         = 'deleted'
    WHERE id = this_id
      AND visibility_change_set_pk = this_visibility_record.visibility_change_set_pk
    RETURNING * INTO this_new_row;
END
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION edge_delete_updates_summaries_trigger_v2() RETURNS trigger AS
$$
DECLARE
    this_deleted_info json;
BEGIN
    IF NEW.visibility_deleted_at IS NOT NULL THEN
        IF NOT EXISTS (SELECT
                       FROM summary_diagram_edges
                       WHERE id = NEW.id
                         AND tenancy_workspace_pk = NEW.tenancy_workspace_pk
                         AND visibility_change_set_pk = NEW.visibility_change_set_pk) THEN
            INSERT INTO summary_diagram_edges
            (id, tenancy_workspace_pk, visibility_change_set_pk, visibility_deleted_at, created_at, updated_at, edge_id,
             from_component_id, from_socket_id, to_component_id, to_socket_id, change_status, created_info,
             deleted_info)
            SELECT id,
                   tenancy_workspace_pk,
                   NEW.visibility_change_set_pk,
                   NEW.visibility_deleted_at,
                   created_at,
                   updated_at,
                   edge_id,
                   from_component_id,
                   from_socket_id,
                   to_component_id,
                   to_socket_id,
                   change_status,
                   created_info,
                   deleted_info
            FROM summary_diagram_edges
            WHERE id = NEW.id
              AND tenancy_workspace_pk = NEW.tenancy_workspace_pk
              AND visibility_change_set_pk = ident_nil_v1();
        END IF;
        this_deleted_info := jsonb_build_object(
                'actor', jsonb_build_object(
                        'pk', COALESCE(NEW.deletion_user_pk, ident_nil_v1()),
                        'kind', 'system',
                        'email', 'system@systeminit.com',
                        'label', 'System Initiative'
                         ),
                'timestamp', NEW.visibility_deleted_at);
        UPDATE summary_diagram_edges
        SET visibility_deleted_at = NEW.visibility_deleted_at,
            change_status         = 'deleted',
            deleted_info          = this_deleted_info
        WHERE id = NEW.id
          AND tenancy_workspace_pk = NEW.tenancy_workspace_pk
          AND visibility_change_set_pk = NEW.visibility_change_set_pk;
    END IF;
    RETURN NEW;
END ;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE TRIGGER edge_delete_trigger
    AFTER INSERT OR UPDATE
    ON edges
    FOR EACH ROW
EXECUTE FUNCTION edge_delete_updates_summaries_trigger_v2();
