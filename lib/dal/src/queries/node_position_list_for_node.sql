SELECT DISTINCT ON (node_positions.id) node_positions.id,
                                       node_positions.visibility_change_set_pk,
                                       node_positions.visibility_deleted_at,
                                       row_to_json(node_positions.*) AS object
FROM node_positions_v1($1, $2) as node_positions
         INNER JOIN node_position_belongs_to_node_v1($1, $2) as node_position_belongs_to_node
                    ON node_position_belongs_to_node.object_id = node_positions.id
                        AND node_position_belongs_to_node.belongs_to_id = $3
ORDER BY node_positions.id,
         node_positions.visibility_change_set_pk DESC,
         node_positions.visibility_deleted_at DESC NULLS FIRST;
