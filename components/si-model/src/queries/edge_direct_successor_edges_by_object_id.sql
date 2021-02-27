SELECT obj AS object
FROM edges
WHERE kind = $1
  AND tail_vertex_object_si_id = $2;
