SELECT row_to_json(func_arguments.*) AS object
FROM func_arguments_v1($1, $2) AS func_arguments
WHERE
    func_arguments.name = $3
    AND func_arguments.func_id = $4
