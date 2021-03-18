SELECT obj as object
FROM key_pairs
WHERE billing_account_id = si_id_to_primary_key_v1($1)
ORDER BY created_lamport_clock DESC
LIMIT 1