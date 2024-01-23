SELECT DISTINCT ON (tail_object_id) tail_object_id as object_id
FROM edges_v1($1, $2) edges
WHERE kind = 'symbolic'
  AND head_object_kind = 'configuration'
  AND head_object_id = $3;
