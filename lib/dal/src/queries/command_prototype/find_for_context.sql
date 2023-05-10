SELECT row_to_json(command_protoypes.*) AS object
FROM command_prototypes_v1($1, $2) AS command_protoypes 
WHERE component_id=$3
  AND schema_variant_id=$4;
