SELECT DISTINCT ON (tail_node_id) tail_node_id as node_id
FROM edges_v1($1, $2) edges
WHERE kind = 'symbolic'
  AND head_object_kind = 'configuration'
  AND head_node_id = $3;
