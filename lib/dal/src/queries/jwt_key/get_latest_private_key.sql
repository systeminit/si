SELECT pk, private_key, nonce
FROM jwt_keys
ORDER BY created_at DESC
LIMIT 1;