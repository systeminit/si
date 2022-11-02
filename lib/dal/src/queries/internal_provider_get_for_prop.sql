SELECT row_to_json(ip.*) AS object
FROM internal_providers_v1($1, $2) AS ip
WHERE prop_id = $3;
