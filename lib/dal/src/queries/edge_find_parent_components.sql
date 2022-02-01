SELECT DISTINCT ON (edges.tail_object_id) edges.tail_object_id
FROM edges
WHERE in_tenancy_v1($1, edges.tenancy_universal, edges.tenancy_billing_account_ids, edges.tenancy_organization_ids,
                    edges.tenancy_workspace_ids)
  AND is_visible_v1($2, edges.visibility_change_set_pk, edges.visibility_edit_session_pk, edges.visibility_deleted)
  AND edges.kind = 'configures'
  AND edges.head_object_kind = 'component'
  AND edges.head_object_id = $3
	ORDER BY 
      edges.tail_object_id DESC,
      edges.id DESC,
      edges.visibility_change_set_pk DESC,
      edges.visibility_edit_session_pk DESC,
      edges.head_node_id DESC,
      edges.head_object_kind DESC,
      edges.head_object_id DESC,
      edges.head_socket_id DESC,
      edges.tail_node_id DESC,
      edges.tail_object_kind DESC,
      edges.tail_socket_id DESC;
