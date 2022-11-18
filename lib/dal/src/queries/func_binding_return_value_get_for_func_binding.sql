SELECT row_to_json(func_binding_return_values.*) as object
FROM func_binding_return_values_v1($1, $2) AS func_binding_return_values
INNER JOIN func_binding_return_value_belongs_to_func_binding_v1($1, $2) AS func_binding_return_value_belongs_to_func_binding
    ON func_binding_return_value_belongs_to_func_binding.object_id = func_binding_return_values.id
WHERE func_binding_return_value_belongs_to_func_binding.belongs_to_id = $3
