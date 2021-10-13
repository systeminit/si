SELECT obj AS object
FROM props
         RIGHT OUTER JOIN schema_props ON schema_props.schema_id = si_id_to_primary_key_v1($1)
WHERE props.id = schema_props.prop_id;