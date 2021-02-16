SELECT jsonb_build_object(
               'value', systems.si_id,
               'label', systems_head.obj ->> 'name'
           ) AS item
FROM systems
         INNER JOIN systems_head
                    ON systems_head.id = systems.id
WHERE systems.workspace_id = si_id_to_primary_key_v1($1);