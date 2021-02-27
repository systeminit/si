SELECT jsonb_build_object(
               'value', entities.si_id,
               'label', entities_head.obj ->> 'name'
           ) AS item
FROM entities
         INNER JOIN entities_head
                    ON entities_head.id = entities.id
WHERE entities.workspace_id = si_id_to_primary_key_v1($1) AND entities.entity_type = 'system';