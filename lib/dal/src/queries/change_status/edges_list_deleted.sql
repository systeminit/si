SELECT row_to_json(e.*) AS object
FROM edges e
WHERE e.id IN (SELECT id
               FROM edges
               WHERE visibility_change_set_pk = ident_nil_v1()
                 AND visibility_deleted_at IS NULL
                 AND in_tenancy_v1($1, tenancy_workspace_pk))

  AND visibility_change_set_pk = $2
  AND visibility_deleted_at IS NOT NULL

  AND in_tenancy_v1($1, tenancy_workspace_pk)
ORDER BY e.id DESC
