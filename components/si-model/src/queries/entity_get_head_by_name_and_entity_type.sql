SELECT entities_head.obj as object
FROM entities_head
         INNER JOIN entities ON entities.id = entities_head.id
WHERE entities_head.obj ->> 'name' = $1
  AND entities.entity_type = $2
  AND entities.workspace_id = si_id_to_primary_key_v1($3);
