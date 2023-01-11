SELECT row_to_json(ip.*) AS object
FROM internal_providers_v1($1, $2) AS ip
INNER JOIN socket_belongs_to_internal_provider_v1($1, $2) AS sbtip
    ON ip.id = sbtip.belongs_to_id
        AND sbtip.object_id = $3;
