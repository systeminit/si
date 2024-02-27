ALTER TABLE summary_diagram_components ADD COLUMN using_default_variant BOOLEAN NOT NULL DEFAULT TRUE;

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
    this_node_id ident,
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
    this_using_default_variant boolean,  -- this field is the change from v1 of this function
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
                                            component_id, display_name, node_id, schema_name,
                                            schema_id, schema_variant_id, schema_variant_name, schema_category,
                                            position, size, color, node_type, change_status, has_resource, created_info,
                                            updated_info, deleted_info, sockets, using_default_variant, child_node_ids)
    VALUES (this_id, this_tenancy_record.tenancy_workspace_pk, this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at,
            this_id, this_display_name, this_node_id, this_schema_name, this_schema_id, this_schema_variant_id,
            this_schema_variant_name,
            this_schema_category, this_position, this_size, this_color, this_node_type, this_change_status,
            this_has_resource, this_created_info, this_updated_info, this_deleted_info,
            this_sockets, this_using_default_variant, jsonb_build_array())
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
    this_schema_id         ident;
    this_schema_name       text;
    this_schema_variant_id ident;
    this_schema_variant_name text;
    this_default_variant_id ident;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    CALL force_component_summary_to_changeset_v2(
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

    --- This is the new addition to the update, calculating the schema/variant info for the component
    --- Normally one would try to express this as a series of inner joins, but that seems to explode
    --- the time this query takes to execute. So we're doing it as a series of selects.
    SELECT cbts.belongs_to_id
    INTO STRICT this_schema_id
    FROM component_belongs_to_schema_v1(this_tenancy, this_visibility) AS cbts
        WHERE cbts.object_id = this_component_id LIMIT 1;

    SELECT schemas.name, schemas.default_schema_variant_id
    INTO STRICT this_schema_name, this_default_variant_id
    FROM schemas_v1(this_tenancy, this_visibility) AS schemas
        WHERE schemas.id = this_schema_id;
    
    SELECT cbtsv.belongs_to_id 
    INTO STRICT this_schema_variant_id 
    FROM component_belongs_to_schema_variant_v1(this_tenancy, this_visibility) AS cbtsv
        WHERE cbtsv.object_id = this_component_id LIMIT 1;

    SELECT schema_variants.name 
    INTO STRICT this_schema_variant_name 
    FROM schema_variants_v1(this_tenancy, this_visibility) AS schema_variants
        WHERE schema_variants.id = this_schema_variant_id;

    UPDATE summary_diagram_components
    SET display_name=this_name,
        color=this_color,
        node_type=this_component_type,
        has_resource=this_has_resource,
        updated_info=this_updated_info,
        visibility_deleted_at = this_deleted_at,
        deleted_info=this_deleted_info,
        change_status=this_change_status,
        schema_id=this_schema_id,
        schema_name=this_schema_name,
        schema_variant_id=this_schema_variant_id,
        schema_variant_name=this_schema_variant_name,
        using_default_variant=COALESCE(this_default_variant_id = this_schema_variant_id, false)
    WHERE component_id = this_component_id
      AND tenancy_workspace_pk = this_tenancy_record.tenancy_workspace_pk
      AND visibility_change_set_pk = this_visibility_record.visibility_change_set_pk
    RETURNING * INTO this_new_row;
END
$$ LANGUAGE PLPGSQL VOLATILE;

-- When a new default schema variant is added, we need to turn "using_default_variant" to false
-- for all components that were using the default variant before the change. We don't have to
-- check anything, so long as this is only run after a new default variant is added, we're good.
CREATE OR REPLACE FUNCTION falsify_using_default_variant_for_components_of_schema_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_schema_id ident
)
RETURNS VOID
AS
$$
DECLARE
    this_summary_row_id ident;
BEGIN
    FOR this_summary_row_id IN
        SELECT summary.id FROM summary_diagram_components_v1(this_tenancy, this_visibility) AS summary
            WHERE summary.schema_id = this_schema_id AND summary.using_default_variant IS true 
    LOOP
        PERFORM update_by_id_v1(
            'summary_diagram_components',
            'using_default_variant',
            this_tenancy,
            this_visibility,
            this_summary_row_id,
            false
        );
    END LOOP;
END
$$ LANGUAGE PLPGSQL;