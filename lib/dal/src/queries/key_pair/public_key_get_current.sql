SELECT row_to_json(key_pairs.*) as object
FROM key_pairs as key_pairs
WHERE key_pairs.workspace_pk = $1
ORDER BY key_pairs.created_lamport_clock DESC
LIMIT 1;
