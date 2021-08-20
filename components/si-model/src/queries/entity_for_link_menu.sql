SELECT COALESCE(entities_edit_session_projection.obj, entities_change_set_projection.obj,
                         entities_head.obj) AS object
FROM entities
         LEFT JOIN entities_edit_session_projection ON entities_edit_session_projection.id = entities.id
    AND entities_edit_session_projection.change_set_id = si_id_to_primary_key_v1($3)
    AND entities_edit_session_projection.edit_session_id = si_id_to_primary_key_v1($4)
         LEFT JOIN entities_change_set_projection ON entities_change_set_projection.id = entities.id
    AND entities_change_set_projection.change_set_id = si_id_to_primary_key_v1($3)
         LEFT JOIN entities_head ON entities_head.id = entities.id
WHERE entities.entity_type = ANY ($1)
  AND entities.workspace_id = si_id_to_primary_key_v1($5)
  AND entities.si_id NOT IN (SELECT edges.head_vertex_object_si_id
                             FROM edges
                             WHERE edges.tail_vertex_object_si_id = $2
                               AND edges.kind = 'component')
