SELECT row_to_json(internal_providers.*) AS object
FROM internal_providers_v1($1, $2) as internal_providers
         LEFT JOIN props_v1($1, $2) as props
                   ON props.id = internal_providers.prop_id
                       AND props.name = $4
         JOIN prop_belongs_to_prop_v1($1, $2) AS prop_belongs_to_prop
              ON prop_belongs_to_prop.object_id = props.id
         JOIN schema_variants_v1($1, $2) AS schema_variants
              ON prop_belongs_to_prop.belongs_to_id = schema_variants.root_prop_id
                    AND schema_variants.id = $3
