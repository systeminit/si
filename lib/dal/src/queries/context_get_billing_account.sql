SELECT billing_accounts.pk
FROM billing_accounts
INNER JOIN workspaces ON workspaces.billing_account_pk = billing_accounts.pk
                         AND workspaces.visibility_deleted_at IS NULL
WHERE workspaces.pk = $1 AND billing_accounts.visibility_deleted_at IS NULL
LIMIT 1;
