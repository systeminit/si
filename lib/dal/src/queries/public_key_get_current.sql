SELECT row_to_json(key_pairs.*) as object
FROM key_pairs_v1($1, $2) as key_pairs
WHERE key_pairs.billing_account_pk = $3
ORDER BY key_pairs.created_lamport_clock DESC
LIMIT 1;
