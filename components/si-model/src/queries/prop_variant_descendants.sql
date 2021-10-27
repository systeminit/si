WITH RECURSIVE search_prop_variant_lineage_descendants(prop_lineage_id, parent_prop_variant_id, child_prop_variant_id,
                                                       change_set_id, edit_session_id, depth, path, cycle) AS (
    SELECT g.id,
           g.parent_prop_variant_id,
           g.child_prop_variant_id,
           g.change_set_id,
           g.edit_session_id,
           1,
           ARRAY [g.parent_prop_variant_id],
           false
    FROM prop_variant_lineage g
    WHERE g.parent_prop_variant_id = si_id_to_primary_key_v1($1)
      AND (
            (g.change_set_id IS NULL AND g.edit_session_id IS NULL)
            OR
            (g.change_set_id IS NOT DISTINCT FROM si_id_to_primary_key_or_null_v1($2) AND g.edit_session_id IS NULL)
            OR
            (g.change_set_id IS NOT DISTINCT FROM
             si_id_to_primary_key_or_null_v1($2)
                AND g.edit_session_id IS NOT DISTINCT FROM si_id_to_primary_key_or_null_v1($3)))
    UNION ALL
    SELECT g.id,
           g.parent_prop_variant_id,
           g.child_prop_variant_id,
           g.change_set_id,
           g.edit_session_id,
           sg.depth + 1,
           path || g.parent_prop_variant_id,
           g.parent_prop_variant_id = ANY (path)
    FROM prop_variant_lineage g,
         search_prop_variant_lineage_descendants sg
    WHERE g.parent_prop_variant_id = sg.child_prop_variant_id
      AND (
            (g.change_set_id IS NULL AND g.edit_session_id IS NULL)
            OR
            (g.change_set_id IS NOT DISTINCT FROM si_id_to_primary_key_or_null_v1($2) AND g.edit_session_id IS NULL)
            OR
            (g.change_set_id IS NOT DISTINCT FROM si_id_to_primary_key_or_null_v1($2)
                AND g.edit_session_id IS NOT DISTINCT FROM si_id_to_primary_key_or_null_v1($3)))
      AND NOT cycle
)
SELECT DISTINCT ON (s.prop_lineage_id) *
FROM search_prop_variant_lineage_descendants s
ORDER BY s.prop_lineage_id, s.change_set_id ASC, s.edit_session_id ASC;
