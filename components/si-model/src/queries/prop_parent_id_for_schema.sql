SELECT parent_id
FROM props_parents
WHERE prop_id = si_id_to_primary_key_v1($1)
  AND schema_id = si_id_to_primary_key_v1($2);