SELECT obj AS object
FROM resources
WHERE obj ->> 'entityId' IN (
    SELECT head_vertex_object_si_id
    FROM edges
    WHERE tail_vertex_object_si_id = $1
      AND kind = 'includes'
      AND head_vertex_object_type = ANY($3))
  AND system_id = si_id_to_primary_key_v1($2);
