SELECT obj as object
FROM entities_projection
WHERE id = si_id_to_primary_key_v1($1)
  AND change_set_id = si_id_to_primary_key_v1($2);