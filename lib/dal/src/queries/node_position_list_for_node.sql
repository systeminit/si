SELECT row_to_json(node_positions.*) AS object
FROM node_positions_v1($1, $2) AS node_positions
INNER JOIN node_position_belongs_to_node_v1($1, $2) AS node_position_belongs_to_node
    ON node_position_belongs_to_node.object_id = node_positions.id
WHERE node_position_belongs_to_node.belongs_to_id = $3
