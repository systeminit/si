SELECT nodes.id AS node_id
FROM nodes_v1($1, $2) as nodes
WHERE nodes.kind = $3;
