SELECT DISTINCT ON (sockets.id) sockets.id,
                                sockets.visibility_change_set_pk,
                                sockets.visibility_deleted_at,
                                row_to_json(sockets.*) AS object

FROM sockets
         INNER JOIN socket_many_to_many_schema_variants
                    ON sockets.id = socket_many_to_many_schema_variants.left_object_id
                        AND in_tenancy_and_visible_v1($1,
                                                      $2,
                                                      socket_many_to_many_schema_variants)
         INNER JOIN component_belongs_to_schema_variant
                    ON socket_many_to_many_schema_variants.right_object_id =
                       component_belongs_to_schema_variant.belongs_to_id
                        AND in_tenancy_and_visible_v1($1,
                                                      $2,
                                                      component_belongs_to_schema_variant)
         INNER JOIN components
                    ON component_belongs_to_schema_variant.object_id = components.id
                        AND in_tenancy_and_visible_v1($1,
                                                      $2,
                                                      components)
                        AND components.id = $3

WHERE sockets.edge_kind = $4
  AND in_tenancy_and_visible_v1($1,
                                $2,
                                sockets)

ORDER BY sockets.id,
         sockets.visibility_change_set_pk DESC,
         sockets.visibility_deleted_at DESC NULLS FIRST;