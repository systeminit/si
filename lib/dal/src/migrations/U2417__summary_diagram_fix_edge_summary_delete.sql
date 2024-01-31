CREATE OR REPLACE FUNCTION force_component_summary_to_changeset_v1(
    this_tenancy_record tenancy_record_v1,
    this_visibility_record visibility_record_v1,
    this_component_id ident,
    this_change_status text,
    OUT object json
) AS
$$
BEGIN
    RAISE WARNING 'Run force_component_summary_to_changeset_v1';

    -- First, we check to see if there is a row already for this change set. If there isn't, we copy the HEAD
    -- row with a few changes.
    IF NOT EXISTS (SELECT
                   FROM summary_diagram_components
                   WHERE component_id = this_component_id
                     AND tenancy_workspace_pk = this_tenancy_record.tenancy_workspace_pk
                     AND visibility_change_set_pk = this_visibility_record.visibility_change_set_pk) THEN
        RAISE WARNING 'Adding component to changeset to delete';

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
          AND visibility_change_set_pk = ident_nil_v1();
    END IF;
END
$$ LANGUAGE PLPGSQL VOLATILE;


CREATE OR REPLACE FUNCTION summary_diagram_component_set_parent_node_id_v2(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_component_id ident,
    this_parent_node_id ident,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           summary_diagram_components%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    PERFORM force_component_summary_to_changeset_v1(
            this_tenancy_record,
            this_visibility_record,
            this_component_id,
            'modified'
            );

    UPDATE summary_diagram_components
    SET parent_node_id=this_parent_node_id
    WHERE component_id = this_component_id
      AND tenancy_workspace_pk = this_tenancy_record.tenancy_workspace_pk
      AND visibility_change_set_pk = this_visibility_record.visibility_change_set_pk
    RETURNING * INTO this_new_row;
END
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION summary_diagram_component_unset_parent_node_id_v2(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_component_id ident,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           summary_diagram_components%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);
    PERFORM force_component_summary_to_changeset_v1(
            this_tenancy_record,
            this_visibility_record,
            this_component_id,
            'modified'
            );


    UPDATE summary_diagram_components
    SET parent_node_id=NULL
    WHERE component_id = this_component_id
      AND tenancy_workspace_pk = this_tenancy_record.tenancy_workspace_pk
      AND visibility_change_set_pk = this_visibility_record.visibility_change_set_pk
    RETURNING * INTO this_new_row;
END
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION restore_edge_by_pk_v1(
    this_pk ident,
    OUT object json
)
AS
$$
BEGIN
    DELETE FROM summary_diagram_edges e WHERE e.pk = this_pk;
END;
$$ LANGUAGE PLPGSQL;
