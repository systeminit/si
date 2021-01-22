SELECT COALESCE(resources_projection.obj, resources_head.obj) AS object
FROM resources
         LEFT OUTER JOIN resources_projection ON (resources_projection.id = resources.id AND
                                                  resources_projection.change_set_id = si_id_to_primary_key_v1($3))
         LEFT OUTER JOIN resources_head ON (resources_head.id = resources.id)
WHERE resources.entity_id = si_id_to_primary_key_v1($1) AND resources.system_id = si_id_to_primary_key_v1($2);
