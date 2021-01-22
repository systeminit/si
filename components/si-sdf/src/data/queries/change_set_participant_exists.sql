SELECT id
FROM change_set_participants
WHERE change_set_id = si_id_to_primary_key_v1($1)
  AND object_si_id = $2;