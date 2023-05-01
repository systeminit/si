SELECT row_to_json(command_protoypes.*) AS object
FROM command_prototypes_v1($1, $2) AS command_protoypes 
WHERE func_id = $3 and schema_variant_id=$4;
