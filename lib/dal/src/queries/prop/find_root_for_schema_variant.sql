SELECT row_to_json(props.*) as object
FROM props_v1($1, $2) AS props
INNER JOIN prop_many_to_many_schema_variants_v1($1, $2) AS prop_many_to_many_schema_variants
    ON prop_many_to_many_schema_variants.left_object_id = props.id
WHERE prop_many_to_many_schema_variants.right_object_id = $3
