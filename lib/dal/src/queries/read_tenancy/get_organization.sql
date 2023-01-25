SELECT DISTINCT ON (organizations.id) organizations.id                 as organization_id,
                                      organizations.billing_account_pk as billing_account_pk
FROM organizations
WHERE organizations.id = $1
      AND is_visible_v1($2, organizations.visibility_change_set_pk, organizations.visibility_deleted_at)
ORDER BY organizations.id DESC,
         organizations.visibility_change_set_pk DESC
LIMIT 1;
