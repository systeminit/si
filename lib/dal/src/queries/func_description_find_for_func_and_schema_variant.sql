SELECT row_to_json(func_descriptions.*) as object
FROM func_descriptions_v1($1, $2) AS func_descriptions
WHERE func_descriptions.func_id = $3
  AND func_descriptions.schema_variant_id = $4
