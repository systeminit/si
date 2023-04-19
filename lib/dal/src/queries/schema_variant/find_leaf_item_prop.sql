SELECT row_to_json(leaf_item_prop.*) AS object
FROM props_v1($1, $2) AS leaf_item_prop
         JOIN prop_belongs_to_prop_v1($1, $2) AS prop_belongs_to_prop
              ON leaf_item_prop.name = $5
                  AND leaf_item_prop.kind = 'object'
                  AND prop_belongs_to_prop.object_id = leaf_item_prop.id
                  AND prop_belongs_to_prop.belongs_to_id IN (
                      SELECT leaf_map_prop.id
                      FROM props_v1($1, $2) AS leaf_map_prop
                               JOIN prop_belongs_to_prop_v1($1, $2) AS prop_belongs_to_prop
                                    ON leaf_map_prop.name = $4
                                        AND leaf_map_prop.kind = 'map'
                                        AND prop_belongs_to_prop.object_id = leaf_map_prop.id
                               JOIN schema_variants_v1($1, $2) as schema_variants
                                    ON prop_belongs_to_prop.belongs_to_id = schema_variants.root_prop_id
                                        AND schema_variants.id = $3
                  )