SELECT entities_head.obj AS object
FROM entities
         LEFT JOIN entities_head ON entities_head.id = entities.id
WHERE entities.id = si_id_to_primary_key_v1($1);