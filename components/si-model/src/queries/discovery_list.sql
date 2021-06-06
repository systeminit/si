SELECT entities_head.obj AS entity, resources.obj AS resource
FROM entities
         INNER JOIN entities_head ON entities.id = entities_head.id
         INNER JOIN resources ON entities.id = resources.entity_id
WHERE entities.entity_type = $2
  AND entities.workspace_id = si_id_to_primary_key_v1($1)