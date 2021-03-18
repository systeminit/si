SELECT true AS authorized
FROM group_capabilities
WHERE (subject = $2 AND action = $3)
   OR (subject = 'any' AND action = 'any')
    AND group_id IN (SELECT group_id FROM group_api_client_members WHERE api_client_id = si_id_to_primary_key_v1($1))
LIMIT 1;