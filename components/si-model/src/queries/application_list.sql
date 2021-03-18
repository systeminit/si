SELECT entities_head.obj as application FROM entities
  INNER JOIN entities_head ON entities.id = entities_head.id
WHERE entities.workspace_id = si_id_to_primary_key_v1($1) AND entities.entity_type = 'application';