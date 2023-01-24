SELECT row_to_json(sockets.*) AS object
FROM sockets_v1($1, $2) as sockets
         JOIN socket_many_to_many_schema_variants_v1($1, $2) as socket_to_schema_variant
              ON sockets.id = socket_to_schema_variant.left_object_id
                  AND sockets.edge_kind = $4
                  AND sockets.kind = 'frame'
         JOIN component_belongs_to_schema_variant_v1($1, $2) as component_belongs_to_schema_variant
              ON component_belongs_to_schema_variant.belongs_to_id = socket_to_schema_variant.right_object_id
         JOIN node_belongs_to_component_v1($1, $2) as node_belongs_to_component
              ON node_belongs_to_component.belongs_to_id = component_belongs_to_schema_variant.object_id
                  AND node_belongs_to_component.object_id = $3;