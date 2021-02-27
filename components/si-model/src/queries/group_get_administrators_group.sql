SELECT obj as object
FROM groups
WHERE name = 'administrators'
  AND billing_account_id = si_id_to_primary_key_v1($1)
LIMIT 1;