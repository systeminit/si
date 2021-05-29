SELECT COALESCE(entities_change_set_projection.obj, entities_head.obj) AS object
FROM entities
         LEFT JOIN entities_change_set_projection ON entities_change_set_projection.id = entities.id
    AND entities_change_set_projection.change_set_id = si_id_to_primary_key_v1($2)
         LEFT JOIN entities_head ON entities_head.id = entities.id
WHERE entities.id = si_id_to_primary_key_v1($1);