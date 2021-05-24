SELECT jsonb_build_object(
               'value', change_sets.si_id,
               'label', change_sets.name
           ) AS item
FROM change_sets
WHERE change_sets.workspace_id = si_id_to_primary_key_v1($1)
  AND change_sets.obj ->> 'status' = 'applied'
ORDER BY change_sets.updated_at;
