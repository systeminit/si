SELECT obj AS object
FROM edges
WHERE kind = $1
  AND head_vertex_node_si_id = $2;
