SELECT row_to_json(props.*) as object
FROM props_v1($1, $2) AS props
         INNER JOIN schema_variants_v1($1, $2) AS schema_variants
                    ON props.id = schema_variants.root_prop_id
WHERE schema_variants.id = $3
