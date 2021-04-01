SELECT qualifications_head.obj AS object
FROM qualifications
LEFT JOIN qualifications_head ON qualifications_head.id = qualifications.id
WHERE qualifications.entity_id = si_id_to_primary_key_v1($1);
