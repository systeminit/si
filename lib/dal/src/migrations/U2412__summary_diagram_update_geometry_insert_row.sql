DROP FUNCTION IF EXISTS summary_diagram_component_update_geometry_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_node_id ident,
    this_position jsonb,
    this_size jsonb,
    OUT object json
);

CREATE OR REPLACE FUNCTION summary_diagram_component_update_geometry_v2(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_node_id ident,
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

    IF NOT EXISTS (SELECT
                   FROM summary_diagram_components
                   WHERE node_id = this_node_id
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
               change_status,
               has_resource,
               created_info,
               updated_info,
               deleted_info,
               sockets,
               parent_node_id,
               child_node_ids
        FROM summary_diagram_components
        WHERE node_id = this_node_id
          AND tenancy_workspace_pk = this_tenancy_record.tenancy_workspace_pk
          AND visibility_change_set_pk = ident_nil_v1();
    END IF;

    UPDATE summary_diagram_components
    SET position = this_position,
        size = this_size
    WHERE node_id = this_node_id
      AND tenancy_workspace_pk = this_tenancy_record.tenancy_workspace_pk
      AND visibility_change_set_pk = this_visibility_record.visibility_change_set_pk
    RETURNING * INTO this_new_row;
END
$$ LANGUAGE PLPGSQL VOLATILE;
