SELECT obj as object FROM entities_head WHERE entities_head.id = si_id_to_primary_key_v1($1)
UNION ALL
SELECT obj as object FROM entities_projection WHERE entities_projection.id = si_id_to_primary_key_v1($1);