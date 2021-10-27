SELECT COALESCE(schema_variants_edit_session_projection.obj, schema_variants_change_set_projection.obj,
                schema_variants_head.obj) AS object
FROM prop_variants_schema_variants
         LEFT JOIN schema_variants_edit_session_projection
                   ON schema_variants_edit_session_projection.id = prop_variants_schema_variants.schema_variant_id
         LEFT JOIN schema_variants_change_set_projection
                   ON schema_variants_change_set_projection.id = prop_variants_schema_variants.schema_variant_id
         LEFT JOIN schema_variants_head ON schema_variants_head.id = prop_variants_schema_variants.schema_variant_id
WHERE prop_variants_schema_variants.prop_variant_id = si_id_to_primary_key_v1($1)
  AND prop_variants_schema_variants.change_set_id IS NOT DISTINCT FROM si_id_to_primary_key_or_null_v1($2)
  AND prop_variants_schema_variants.edit_session_id IS NOT DISTINCT FROM si_id_to_primary_key_or_null_v1($3)
  AND prop_variants_schema_variants.deleted = false;