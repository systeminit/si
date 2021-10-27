SELECT COALESCE(prop_variants_edit_session_projection.obj, prop_variants_change_set_projection.obj,
                prop_variants_head.obj) AS object
FROM prop_variant_lineage
         LEFT JOIN prop_variants_edit_session_projection
                   ON prop_variants_edit_session_projection.id = prop_variant_lineage.parent_prop_variant_id
         LEFT JOIN prop_variants_change_set_projection
                   ON prop_variants_change_set_projection.id = prop_variant_lineage.parent_prop_variant_id
         LEFT JOIN prop_variants_head ON prop_variants_head.id = prop_variant_lineage.parent_prop_variant_id
WHERE prop_variant_lineage.child_prop_variant_id = si_id_to_primary_key_v1($1)
  AND prop_variant_lineage.change_set_id IS NOT DISTINCT FROM si_id_to_primary_key_or_null_v1($2)
  AND prop_variant_lineage.edit_session_id IS NOT DISTINCT FROM si_id_to_primary_key_or_null_v1($3)
  AND prop_variant_lineage.deleted = false;