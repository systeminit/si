SELECT row_to_json(props.*) AS object
FROM props_v1($1, $2) AS props
INNER JOIN component_belongs_to_schema_variant_v1($1, $2) bt ON bt.belongs_to_id = props.schema_variant_id
WHERE props.validation_format IS NOT NULL AND bt.object_id = $3
