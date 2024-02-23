SELECT DISTINCT ON (head_component_id) head_component_id as object_id
FROM edges_v1($1, $2) AS edges
WHERE kind = 'symbolic'
  AND tail_component_id = $3;
