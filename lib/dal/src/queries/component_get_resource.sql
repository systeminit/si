SELECT row_to_json(fbrv.*) AS object
FROM resource_resolvers_v1($1, $2) AS rr
INNER JOIN func_binding_return_values_v1($1, $2) AS fbrv
    ON fbrv.func_binding_id = rr.func_binding_id
WHERE
    rr.component_id = $3
