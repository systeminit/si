SELECT row_to_json(props.*) AS object
FROM props_v1($1, $2) AS props
WHERE props.schema_variant_id = $3 AND props.path = $4