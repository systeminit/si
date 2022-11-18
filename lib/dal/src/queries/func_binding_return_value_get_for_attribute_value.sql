SELECT row_to_json(fbrv.*) as object
FROM func_binding_return_values_v1($1, $2) AS fbrv
INNER JOIN attribute_values_v1($1, $2) AS av
    ON fbrv.id = av.func_binding_return_value_id
WHERE av.id = $3;
