SELECT row_to_json(schema_ui_menus.*) AS object
FROM schema_ui_menus_v1($1, $2) AS schema_ui_menus
INNER JOIN schema_ui_menu_belongs_to_schema_v1($1, $2) AS schema_ui_menu_belongs_to_schema
    ON schema_ui_menus.id = schema_ui_menu_belongs_to_schema.object_id
INNER JOIN schemas_v1($1, $2) AS schemas
    ON schema_ui_menu_belongs_to_schema.belongs_to_id = schemas.id
        AND schemas.id = $3
WHERE schema_ui_menus.diagram_kind = $4
