SELECT row_to_json(sockets.*) AS object
FROM sockets_v1($1, $2) as sockets
  JOIN socket_belongs_to_external_provider_v1($1, $2) as sbtep 
    ON sbtep.object_id = sockets.id
      AND sbtep.belongs_to_id = $3;
