SELECT DISTINCT ON (func_arguments.name)
    func_arguments.name,
    row_to_json(func_arguments.*) as func_argument_object,
    row_to_json(apa.*)            AS prototype_argument_object
FROM func_arguments_v1($1, $2) AS func_arguments
LEFT JOIN attribute_prototype_arguments_v1($1, $2) AS apa
    ON func_arguments.id = apa.func_argument_id
        AND apa.attribute_prototype_id = $4
WHERE func_arguments.func_id = $3
ORDER BY func_arguments.name
