SELECT json_build_object(
               'encrypted_secret', row_to_json(secret),
               'func', row_to_json(func.*)) AS object
FROM components_v1($1, $2) c -- Starting from a component id
         JOIN component_belongs_to_schema_variant_v1($1, $2) cbtsv ON cbtsv.object_id = c.id
    -- Get all of its props below root/secrets
         JOIN props_v1($1, $2) secret_prop
              ON secret_prop.schema_variant_id = cbtsv.belongs_to_id
                  AND secret_prop.path LIKE 'rootsecrets%'
                  AND -- Sanity check, value should always be set for secret prop
                 secret_prop.widget_options -> 0 ->> 'value' IS NOT NULL
-- Get  secret defining schema variant id by joining with root/secrets and root/secret_definition
         JOIN props_v1($1, $2) base_secret_prop
              ON secret_prop.widget_options -> 0 ->> 'value' = base_secret_prop.name
         JOIN props_v1($1, $2) secret_defining_prop
              ON secret_defining_prop.path = 'rootsecret_definition'
                  AND secret_defining_prop.schema_variant_id = base_secret_prop.schema_variant_id
    -- Use auth prototypes bound to secret defining schema variant to get functions
         JOIN authentication_prototypes_v1($1, $2) proto
              ON proto.schema_variant_id = secret_defining_prop.schema_variant_id
         JOIN funcs_v1($1, $2) func ON proto.func_id = func.id
    -- That was half of it, now we join from secret prop to get secret values. We can skip function execution if unset
         JOIN attribute_values_v1($1, $2) av
              ON av.attribute_context_prop_id = secret_prop.id
                  AND av.attribute_context_component_id = c.id
         JOIN func_binding_return_values_v1($1, $2) fbrv
              ON av.func_binding_return_value_id = fbrv.id AND fbrv.value IS NOT NULL
    -- Then we join encrypted secrets by converting the fbrv value (which is a json b) into text and then ident
         JOIN encrypted_secrets_v1($1, $2) secret ON secret.id = (fbrv.value #>> '{}')::ident
where c.id = $3;





