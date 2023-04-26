SELECT row_to_json(sockets.*) AS object
FROM sockets_v1($1, $2) as sockets
  JOIN socket_belongs_to_internal_provider_v1($1, $2) as sbtip 
    ON sbtip.object_id = sockets.id
      AND sbtip.belongs_to_id = $3;
