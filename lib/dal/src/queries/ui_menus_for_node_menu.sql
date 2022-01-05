SELECT schema_ui_menus.id                       AS id,
       schema_ui_menus.visibility_change_set_pk AS visibility_change_set_pk,
       schema_ui_menus.visibility_edit_session_pk AS visibility_edit_session_pk,
       row_to_json(schema_ui_menus.*) AS object
FROM component_belongs_to_schema
         INNER JOIN schema_ui_menu_root_schematic_many_to_many_schematic
                    ON component_belongs_to_schema.belongs_to_id = schema_ui_menu_root_schematic_many_to_many_schematic.right_object_id
                        AND is_visible_v1($2
                           , schema_ui_menu_root_schematic_many_to_many_schematic.visibility_change_set_pk
                           , schema_ui_menu_root_schematic_many_to_many_schematic.visibility_edit_session_pk
                           , schema_ui_menu_root_schematic_many_to_many_schematic.visibility_deleted)
         INNER JOIN schema_ui_menus
                    ON schema_ui_menus.id = schema_ui_menu_root_schematic_many_to_many_schematic.left_object_id
                        AND schema_ui_menus.schematic_kind = $4
                        AND is_visible_v1($2
                           , schema_ui_menus.visibility_change_set_pk
                           , schema_ui_menus.visibility_edit_session_pk
                           , schema_ui_menus.visibility_deleted)
WHERE component_belongs_to_schema.object_id = $3
  AND in_tenancy_v1($1
    , component_belongs_to_schema.tenancy_universal
    , component_belongs_to_schema.tenancy_billing_account_ids
    , component_belongs_to_schema.tenancy_organization_ids
    , component_belongs_to_schema.tenancy_workspace_ids)
  AND is_visible_v1($2
    , component_belongs_to_schema.visibility_change_set_pk
    , component_belongs_to_schema.visibility_edit_session_pk
    , component_belongs_to_schema.visibility_deleted)
;

