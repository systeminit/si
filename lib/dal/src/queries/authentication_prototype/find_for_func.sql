SELECT row_to_json(prototypes.*) AS object
FROM authentication_prototypes_v1($1, $2) AS prototypes
WHERE prototypes.func_id = $3
