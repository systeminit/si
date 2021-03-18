SELECT resources_head.obj AS object
FROM resources
         INNER JOIN resources_head ON (resources_head.id = resources.id)
WHERE resources.entity_id = si_id_to_primary_key_v1($1)
  AND resources.system_id = si_id_to_primary_key_v1($2)