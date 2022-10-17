SELECT DISTINCT ON (organizations.id) organizations.id                                      as organization_id,
                                      organization_belongs_to_billing_account.belongs_to_id as billing_account_id
FROM organizations
         INNER JOIN organization_belongs_to_billing_account
                    on organization_belongs_to_billing_account.object_id = organizations.id
                        AND is_visible_v1(
                               $2,
                               organization_belongs_to_billing_account.visibility_change_set_pk,
                               organization_belongs_to_billing_account.visibility_deleted_at)

WHERE organizations.id = $1
  AND is_visible_v1($2, organizations.visibility_change_set_pk, organizations.visibility_deleted_at)
ORDER BY organizations.id DESC,
         organizations.visibility_change_set_pk DESC
LIMIT 1;
