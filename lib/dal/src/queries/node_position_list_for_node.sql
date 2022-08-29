SELECT DISTINCT ON (node_positions.id) node_positions.id,
                                       node_positions.visibility_change_set_pk,
                                       node_positions.visibility_deleted_at,
                                       row_to_json(node_positions.*) AS object
FROM node_positions
         INNER JOIN node_position_belongs_to_node
                    ON node_position_belongs_to_node.object_id = node_positions.id
                        AND in_tenancy_and_visible_v1($1, $2, node_position_belongs_to_node)
                        AND node_position_belongs_to_node.belongs_to_id = $3

WHERE in_tenancy_and_visible_v1($1, $2, node_positions)
  AND CASE
          WHEN $4::bigint IS NULL THEN node_positions.system_id IS NULL
          ELSE node_positions.system_id = $4::bigint
    END

ORDER BY node_positions.id,
         node_positions.visibility_change_set_pk DESC,
         node_positions.visibility_deleted_at DESC NULLS FIRST;
