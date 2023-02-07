SELECT password
FROM users
INNER JOIN workspaces ON workspaces.pk = $2
INNER JOIN organizations ON organizations.pk = workspaces.organization_pk
WHERE users.pk = $1 AND users.billing_account_pk = organizations.billing_account_pk;
