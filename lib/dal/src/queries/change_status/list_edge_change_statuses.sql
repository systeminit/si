(SELECT DISTINCT ON (e.id) e.id, 'added' as status
 FROM edges e
 WHERE e.id NOT IN (SELECT id
                    FROM edges
                    WHERE visibility_change_set_pk = ident_nil_v1()
                      AND visibility_deleted_at IS NULL
                      AND in_tenancy_v1($1,
                                        tenancy_universal,
                                        tenancy_billing_account_ids,
                                        tenancy_organization_ids,
                                        tenancy_workspace_ids))

   AND visibility_change_set_pk = $2

   -- Ensure they are not deleted
   AND visibility_deleted_at IS NULL

   AND in_tenancy_v1($1,
                     tenancy_universal,
                     tenancy_billing_account_ids,
                     tenancy_organization_ids,
                     tenancy_workspace_ids)
 ORDER BY e.id DESC,
          e.tenancy_universal)
UNION
(SELECT DISTINCT ON (e.id) e.id, 'delete' as status
 FROM edges e
 WHERE e.id IN (SELECT id
                FROM edges
                WHERE visibility_change_set_pk = ident_nil_v1()
                  AND visibility_deleted_at IS NULL
                  AND in_tenancy_v1($1,
                                    tenancy_universal,
                                    tenancy_billing_account_ids,
                                    tenancy_organization_ids,
                                    tenancy_workspace_ids))

   AND visibility_change_set_pk = $2

   AND visibility_deleted_at IS NOT NULL

   AND in_tenancy_v1($1,
                     tenancy_universal,
                     tenancy_billing_account_ids,
                     tenancy_organization_ids,
                     tenancy_workspace_ids)
 ORDER BY e.id DESC,
          e.tenancy_universal)
