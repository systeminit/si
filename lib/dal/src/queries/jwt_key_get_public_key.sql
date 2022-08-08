SELECT public_key
FROM jwt_keys
WHERE pk = $1;