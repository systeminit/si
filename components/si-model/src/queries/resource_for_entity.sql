SELECT obj AS object
FROM resources
WHERE entity_id = si_id_to_primary_key_v1($1);
