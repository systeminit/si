SELECT row_to_json(ip.*) AS object
FROM internal_providers_v1($1, $2) AS ip
JOIN socket_belongs_to_internal_provider_v1($1, $2) AS sbtip
     ON ip.id = sbtip.belongs_to_id
JOIN sockets_v1($1, $2) AS s
     ON s.id = sbtip.object_id
WHERE ip.prop_id = ident_nil_v1() 
     AND (
          $3::ident IS NULL 
          OR ip.schema_variant_id = $3::ident
     );