SELECT obj as object
FROM systems_projection
WHERE id = si_id_to_primary_key_v1($1)
AND change_set_id = si_id_to_primary_key_v1($2);