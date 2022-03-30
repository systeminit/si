SELECT DISTINCT ON (node_positions.id) node_positions.id,
                              node_positions.visibility_change_set_pk,
                              node_positions.visibility_edit_session_pk,
                              row_to_json(node_positions.*) AS object
FROM node_positions
INNER JOIN node_position_belongs_to_node AS bt ON bt.object_id = node_positions.id
WHERE in_tenancy_v1($1, node_positions.tenancy_universal, node_positions.tenancy_billing_account_ids, node_positions.tenancy_organization_ids,
                    node_positions.tenancy_workspace_ids)
  AND is_visible_v1($2, node_positions.visibility_change_set_pk, node_positions.visibility_edit_session_pk, node_positions.visibility_deleted)
  AND CASE
    WHEN $3::bigint IS NULL THEN node_positions.system_id IS NULL
    ELSE node_positions.system_id = $3::bigint
  END
  AND node_positions.root_node_id = $4
  AND bt.belongs_to_id = $5
ORDER BY id, visibility_change_set_pk DESC, visibility_edit_session_pk DESC;
