SELECT billing_accounts.pk
FROM billing_accounts
INNER JOIN organizations ON organizations.billing_account_pk = billing_accounts.pk
                            AND organizations.visibility_deleted_at IS NULL
INNER JOIN workspaces ON workspaces.organization_pk = organizations.pk
                         AND workspaces.visibility_deleted_at IS NULL
WHERE workspaces.pk = $1 AND billing_accounts.visibility_deleted_at IS NULL
LIMIT 1;
