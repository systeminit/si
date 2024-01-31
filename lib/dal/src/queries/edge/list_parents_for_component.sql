SELECT DISTINCT ON (head_object_id) head_object_id as object_id
FROM edges_v1($1, $2) AS edges
WHERE kind = 'symbolic'
  AND tail_object_kind = 'configuration'
  AND tail_object_id = $3;
