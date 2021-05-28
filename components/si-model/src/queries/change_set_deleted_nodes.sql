SELECT id
FROM entities_change_set_projection
WHERE entities_change_set_projection.change_set_id = si_id_to_primary_key_v1($1)
  AND entities_change_set_projection.obj -> 'siStorable' -> 'deleted' = 'true'::jsonb;