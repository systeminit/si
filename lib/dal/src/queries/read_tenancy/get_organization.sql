SELECT DISTINCT ON (organizations.pk) organizations.pk                 as organization_pk,
                                      organizations.billing_account_pk as billing_account_pk
FROM organizations
WHERE organizations.pk = $1 AND organizations.visibility_deleted_at IS NULL
ORDER BY organizations.pk DESC
LIMIT 1;
