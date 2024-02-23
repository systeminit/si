SELECT DISTINCT ON (tail_component_id) tail_component_id as object_id
FROM edges_v1($1, $2) edges
WHERE kind = 'symbolic'
  AND head_component_id = $3;
