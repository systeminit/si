SELECT row_to_json(ap.*) AS object
FROM attribute_prototypes_v1($1, $2) AS ap
WHERE func_id = $3;
