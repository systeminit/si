SELECT COALESCE(resources_edit_session_projection.obj, resources_change_set_projection.obj, resources_head.obj) AS object
FROM resources
LEFT JOIN resources_edit_session_projection ON resources_edit_session_projection.id = resources.id
                                            AND resources_edit_session_projection.change_set_id = si_id_to_primary_key_v1($2)
                                            AND resources_edit_session_projection.edit_session_id = si_id_to_primary_key_v1($3)
LEFT JOIN resources_change_set_projection ON resources_change_set_projection.id = resources.id
                                         AND resources_change_set_projection.change_set_id = si_id_to_primary_key_v1($2)
LEFT JOIN resources_head ON resources_head.id = resources.id
WHERE resources.entity_id = si_id_to_primary_key_v1($1);
