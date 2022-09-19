SELECT DISTINCT ON (schema_ui_menus.id) schema_ui_menus.id,
                                        schema_ui_menus.visibility_change_set_pk,
                                        schema_ui_menus.visibility_deleted_at,
                                        row_to_json(schema_ui_menus.*) AS object

FROM schema_ui_menus
         INNER JOIN schema_ui_menu_belongs_to_schema
                    ON schema_ui_menus.id = schema_ui_menu_belongs_to_schema.object_id
                        AND in_tenancy_and_visible_v1($1, $2, schema_ui_menu_belongs_to_schema)
         INNER JOIN schemas
                    ON schema_ui_menu_belongs_to_schema.belongs_to_id = schemas.id
                        AND in_tenancy_and_visible_v1($1, $2, schemas)
                        AND schemas.id = $3

WHERE schema_ui_menus.diagram_kind = $4
  AND in_tenancy_and_visible_v1($1, $2, schema_ui_menus)

ORDER BY schema_ui_menus.id DESC,
         schema_ui_menus.visibility_change_set_pk DESC,
         schema_ui_menus.visibility_deleted_at DESC NULLS FIRST;