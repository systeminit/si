SELECT password
FROM users
WHERE pk = $1 AND billing_account_pk = $2;
