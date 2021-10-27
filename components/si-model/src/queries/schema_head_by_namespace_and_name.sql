SELECT schemas_head.obj AS object
FROM schemas
         LEFT JOIN schemas_head ON schemas.id = schemas_head.id
WHERE schemas.namespace = $1
  AND schemas.name = $2;

SELECT row_to_json(schemas) from schemas WHERE schemas.namespace = $1 AND schemas.name $2;