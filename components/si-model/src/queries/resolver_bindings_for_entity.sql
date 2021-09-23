SELECT resolver_bindings.obj AS resolver_binding, props.obj AS prop
FROM resolver_bindings
         LEFT OUTER JOIN props ON props.schema_id = resolver_bindings.schema_id AND resolver_bindings.prop_id = props.id
WHERE resolver_bindings.schema_id = si_id_to_primary_key_v1($1)
   OR resolver_bindings.entity_id = si_id_to_primary_key_v1($2);