CREATE OR REPLACE FUNCTION summary_diagram_component_update_v1(
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
    IF this_deleted_at IS NOT NULL then
        this_change_status := 'deleted';
    ELSE
        this_change_status := 'modified';
    END IF;

    -- First, we check to see if there is a row already for this change set. If there isn't, we copy the HEAD
    -- row with a few changes.
    IF NOT EXISTS (SELECT
                   FROM summary_diagram_components
                   WHERE component_id = this_component_id
                     AND tenancy_workspace_pk = this_tenancy_record.tenancy_workspace_pk
                     AND visibility_change_set_pk = this_visibility_record.visibility_change_set_pk) THEN
        INSERT INTO summary_diagram_components (id, tenancy_workspace_pk, visibility_change_set_pk,
                                                visibility_deleted_at, created_at, component_id,
                                                display_name, node_id, schema_name,
                                                schema_id, schema_variant_id,
                                                schema_variant_name, schema_category, position, size, color, node_type,
                                                change_status, has_resource, created_info, updated_info, deleted_info,
                                                sockets, parent_node_id, child_node_ids)
        SELECT id,
               tenancy_workspace_pk,
               this_visibility_record.visibility_change_set_pk AS visibility_change_set_pk,
               this_visibility_record.visibility_deleted_at    AS visibility_deleted_at,
               created_at,
               component_id,
               display_name,
               node_id,
               schema_name,
               schema_id,
               schema_variant_id,
               schema_variant_name,
               schema_category,
               position,
               size,
               color,
               node_type,
               this_change_status,
               has_resource,
               created_info,
               updated_info,
               deleted_info,
               sockets,
               parent_node_id,
               child_node_ids
        FROM summary_diagram_components
        WHERE id = this_component_id
          AND tenancy_workspace_pk = this_tenancy_record.tenancy_workspace_pk
          AND visibility_change_set_pk = '00000000000000000000000000';
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
    -- AND in_tenancy_and_visible_v1(this_tenancy, this_visibility, summary_diagram_components)
    RETURNING * INTO this_new_row;
END
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION summary_diagram_component_change_status_trigger_v1() RETURNS trigger AS
$$
BEGIN
    IF NEW.change_status != 'deleted' THEN
        NEW.change_status := 'unmodified';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE PLPGSQL;

