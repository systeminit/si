SELECT row_to_json(apa.*) AS object
FROM attribute_prototype_arguments_v1($1, $2) AS apa
WHERE apa.func_argument_id = $3
