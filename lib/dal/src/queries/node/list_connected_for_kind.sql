SELECT nodes.id AS node_id
FROM nodes_v1($1, $2) as nodes
         LEFT JOIN edges_v1($1, $2) as edges
                   ON edges.head_node_id = nodes.id
                       OR edges.tail_node_id = nodes.id
WHERE nodes.kind = $3
  AND (edges.tail_node_id = nodes.id OR edges.head_node_id = nodes.id)