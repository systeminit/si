SELECT DISTINCT ON (components.id) components.id,
                                   components.visibility_change_set_pk,
                                   components.visibility_deleted_at,
                                   row_to_json(components.*) AS object

FROM components
         INNER JOIN node_belongs_to_component
                    ON components.id = node_belongs_to_component.belongs_to_id
                        AND in_tenancy_and_visible_v1($1, $2, node_belongs_to_component)
                        AND node_belongs_to_component.object_id = $3
WHERE in_tenancy_and_visible_v1($1, $2, components)

ORDER BY components.id,
         components.visibility_change_set_pk DESC,
         components.visibility_deleted_at DESC NULLS FIRST;
