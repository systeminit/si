(SELECT row_to_json(funcs.*) AS object
 FROM props_v1($1, $2) as props
          JOIN attribute_prototypes_v1($1, $2) ap
               ON ap.attribute_context_prop_id = props.id
          JOIN funcs_v1($1, $2) funcs
               ON funcs.id = ap.func_id
 WHERE props.id in
       (WITH RECURSIVE recursive_props
                           AS
                           (SELECT root_prop_id AS prop_id
                            FROM schema_variants_v1($1, $2) AS schema_variants
                            WHERE schema_variants.id = $3
                            UNION ALL
                            SELECT pbp.object_id as prop_id
                            FROM prop_belongs_to_prop_v1($1, $2) AS pbp
                                     JOIN recursive_props
                                          ON pbp.belongs_to_id = recursive_props.prop_id)
        SELECT prop_id
        FROM recursive_props)
   AND funcs.code_sha256 != '0'
   -- we don't want component specific functions
   AND ap.attribute_context_component_id = ident_nil_v1())

UNION ALL

(SELECT row_to_json(funcs.*) AS object
 FROM internal_providers_v1($1, $2) ip
          JOIN attribute_prototypes_v1($1, $2) ap
               ON ap.attribute_context_internal_provider_id = ip.id
          JOIN funcs_v1($1, $2) funcs
               ON ap.func_id = funcs.id
 WHERE funcs.code_sha256 != '0'
   AND ip.schema_variant_id = $3
   AND ap.attribute_context_component_id = ident_nil_v1())

UNION ALL

(SELECT row_to_json(funcs.*) AS object
 FROM external_providers_v1($1, $2) ep
          JOIN attribute_prototypes_v1($1, $2) ap
               ON ap.attribute_context_external_provider_id = ep.id
          JOIN funcs_v1($1, $2) funcs
               ON ap.func_id = funcs.id
 WHERE funcs.code_sha256 != '0'
   AND ep.schema_variant_id = $3
   AND ap.attribute_context_component_id = ident_nil_v1())

UNION ALL

(SELECT row_to_json(funcs.*) AS object
 FROM validation_prototypes_v1($1, $2) vp
          JOIN funcs_v1($1, $2) funcs
               ON vp.func_id = funcs.id
 WHERE vp.schema_variant_id = $3
   AND funcs.code_sha256 != '0')

UNION ALL

(SELECT row_to_json(funcs.*) AS object
 FROM workflow_prototypes_v1($1, $2) wp
          JOIN funcs_v1($1, $2) funcs
               ON funcs.id = wp.func_id
 WHERE wp.schema_variant_id = $3
   AND funcs.code_sha256 != '0')

