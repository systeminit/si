SELECT row_to_json(key_pairs.*) AS object
FROM key_pairs
WHERE key_pairs.pk = $1 AND key_pairs.visibility_deleted_at IS NULL
