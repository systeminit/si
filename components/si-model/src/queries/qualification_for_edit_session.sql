SELECT COALESCE(qualifications_edit_session_projection.obj, qualifications_change_set_projection.obj, qualifications_head.obj) AS object
FROM qualifications
LEFT JOIN qualifications_edit_session_projection ON qualifications_edit_session_projection.id = qualifications.id
                                            AND qualifications_edit_session_projection.change_set_id = si_id_to_primary_key_v1($2)
                                            AND qualifications_edit_session_projection.edit_session_id = si_id_to_primary_key_v1($3)
LEFT JOIN qualifications_change_set_projection ON qualifications_change_set_projection.id = qualifications.id
                                         AND qualifications_change_set_projection.change_set_id = si_id_to_primary_key_v1($2)
LEFT JOIN qualifications_head ON qualifications_head.id = qualifications.id
WHERE qualifications.entity_id = si_id_to_primary_key_v1($1);