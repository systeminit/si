SELECT DISTINCT ON (node_positions.id) node_positions.id,
                              node_positions.visibility_change_set_pk,
                              node_positions.visibility_edit_session_pk,
                              row_to_json(node_positions.*) AS object
FROM node_positions
INNER JOIN node_position_belongs_to_node AS bt ON bt.object_id = node_positions.id
WHERE in_tenancy_v1($1, node_positions.tenancy_universal, node_positions.tenancy_billing_account_ids, node_positions.tenancy_organization_ids,
                    node_positions.tenancy_workspace_ids)
  AND is_visible_v1($2, node_positions.visibility_change_set_pk, node_positions.visibility_edit_session_pk, node_positions.visibility_deleted)
  AND node_positions.schematic_kind = $3
  AND node_positions.system_id = $4
  AND node_positions.root_node_id = $5
  AND bt.belongs_to_id = $6
ORDER BY id, visibility_change_set_pk DESC, visibility_edit_session_pk DESC
LIMIT 1;
