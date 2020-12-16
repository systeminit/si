SELECT obj as object
FROM users
WHERE email = $1
  AND billing_account_id = si_id_to_primary_key_v1($2)
LIMIT 1