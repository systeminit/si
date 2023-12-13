SELECT DISTINCT ON (head_object_id) head_object_id
FROM edges_v1($1, $2) AS edges
WHERE kind = 'configuration'
  AND tail_object_kind = 'configuration'
  AND tail_object_id = $3
ORDER BY head_object_id DESC,
         id DESC,
         tail_node_id DESC,
         tail_object_kind DESC,
         tail_object_id DESC,
         tail_socket_id DESC,
         tail_node_id DESC,
         tail_object_kind DESC,
         tail_socket_id DESC;
