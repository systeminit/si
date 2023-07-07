ALTER table schema_variant_definitions add column func_id ident NOT NULL DEFAULT ident_nil_v1();
ALTER TABLE schema_variant_definitions ADD COLUMN schema_variant_id ident;

drop function schema_variant_definition_create_v1;

CREATE OR REPLACE FUNCTION schema_variant_definition_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_name text,
    this_menu_name text,
    this_category text,
    this_link text,
    this_color text,
    this_component_kind text,
    this_func_id ident,
    this_description text,
    OUT object json) AS
    $$
DECLARE
this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           schema_variant_definitions%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

INSERT INTO schema_variant_definitions (
    tenancy_workspace_pk, visibility_change_set_pk,
    name, menu_name, category, link, func_id, color, component_kind,
    description
) VALUES (
             this_tenancy_record.tenancy_workspace_pk,
             this_visibility_record.visibility_change_set_pk,
             this_name,
             this_menu_name, this_category, this_link, this_func_id, this_color,
             this_component_kind, this_description
         )
    RETURNING * INTO this_new_row;

object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

