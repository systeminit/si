SELECT row_to_json(internal_providers.*) AS object

FROM internal_providers_v1($1, $2) as internal_providers
         LEFT JOIN props_v1($1, $2) as props
                   ON props.id = internal_providers.prop_id
         JOIN prop_belongs_to_prop_v1($1, $2) AS prop_belongs_to_prop
              ON prop_belongs_to_prop.object_id = props.id

WHERE props.name = $4
  AND prop_belongs_to_prop.belongs_to_id IN (
    SELECT prop_many_to_many_schema_variants.left_object_id AS root_prop_id
    FROM prop_many_to_many_schema_variants_v1($1, $2) AS prop_many_to_many_schema_variants
    WHERE prop_many_to_many_schema_variants.right_object_id = $3
)
