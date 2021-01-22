SELECT COALESCE(entities_head.obj, entities_base.obj) AS object
FROM entities
         LEFT OUTER JOIN entities_head ON (entities_head.id = entities.id)
         LEFT OUTER JOIN entities_base
                         ON (entities_base.id = entities.id AND
                             entities_base.change_set_id = si_id_to_primary_key_v1($2))
WHERE entities.id = si_id_to_primary_key_v1($1);
