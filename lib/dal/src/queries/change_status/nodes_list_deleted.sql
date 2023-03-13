SELECT row_to_json(n.*) AS object
FROM nodes n
WHERE n.id IN (SELECT nodes.id
               FROM nodes
               INNER JOIN node_belongs_to_component bt ON bt.object_id = nodes.id
               INNER JOIN components ON components.id = bt.belongs_to_id
               WHERE nodes.visibility_change_set_pk = ident_nil_v1()
                 AND (nodes.visibility_deleted_at IS NULL OR components.needs_destroy)
                 AND in_tenancy_v1($1, nodes.tenancy_workspace_pk))
  AND visibility_change_set_pk = $2
  AND visibility_deleted_at IS NOT NULL
  AND in_tenancy_v1($1, tenancy_workspace_pk)
ORDER BY n.id DESC
