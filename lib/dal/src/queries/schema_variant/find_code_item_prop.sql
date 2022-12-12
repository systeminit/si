SELECT row_to_json(code_map_item_prop.*) AS object
FROM props_v1($1, $2) AS code_map_item_prop
         JOIN prop_belongs_to_prop_v1($1, $2) AS prop_belongs_to_prop
              ON code_map_item_prop.name = 'codeItem'
                  AND prop_belongs_to_prop.object_id = code_map_item_prop.id
                  AND prop_belongs_to_prop.belongs_to_id IN (
                      SELECT code_map_prop.id
                      FROM props_v1($1, $2) AS code_map_prop
                               JOIN prop_belongs_to_prop_v1($1, $2) AS prop_belongs_to_prop
                                    ON code_map_prop.name = 'code'
                                        AND prop_belongs_to_prop.object_id = code_map_prop.id
                                        AND prop_belongs_to_prop.belongs_to_id IN (
                                            SELECT prop_many_to_many_schema_variants.left_object_id AS root_prop_id
                                            FROM prop_many_to_many_schema_variants_v1($1, $2) AS prop_many_to_many_schema_variants
                                            WHERE prop_many_to_many_schema_variants.right_object_id = $3
                                        )
                  )