SELECT DISTINCT ON (tail_object_id) tail_object_id
FROM edges_v1($1, $2) AS edges
WHERE kind = 'configuration'
  AND head_object_kind = 'configuration'
  AND head_object_id = $3
ORDER BY tail_object_id DESC,
         id DESC,
         head_node_id DESC,
         head_object_kind DESC,
         head_object_id DESC,
         head_socket_id DESC,
         tail_node_id DESC,
         tail_object_kind DESC,
         tail_socket_id DESC;
