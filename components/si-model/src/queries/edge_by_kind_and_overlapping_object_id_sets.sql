SELECT obj AS object
FROM edges
WHERE kind = $1
  AND head_vertex_object_si_id = ANY ($2)
  AND tail_vertex_object_si_id = ANY ($2);