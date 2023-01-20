SELECT row_to_json(props.*) AS object
FROM props_v1($1, $2) as props
         JOIN prop_belongs_to_prop_v1($1, $2) AS si_child_prop_belongs_to_si_prop
              ON si_child_prop_belongs_to_si_prop.object_id = props.id
         JOIN props_v1($1, $2) as si_prop
              ON si_child_prop_belongs_to_si_prop.belongs_to_id = si_prop.id
                  AND si_prop.name = 'si'
         JOIN prop_belongs_to_prop_v1($1, $2) AS si_prop_belongs_to_root_prop
              ON si_child_prop_belongs_to_si_prop.belongs_to_id = si_prop_belongs_to_root_prop.object_id
                  AND si_prop_belongs_to_root_prop.belongs_to_id IN (
                      SELECT prop_many_to_many_schema_variants.left_object_id AS root_prop_id
                      FROM prop_many_to_many_schema_variants_v1($1, $2) AS prop_many_to_many_schema_variants
                      WHERE prop_many_to_many_schema_variants.right_object_id = $3
                  )
