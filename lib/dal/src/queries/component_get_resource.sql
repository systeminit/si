SELECT DISTINCT ON (rr.id)
    row_to_json(fbrv.*) AS object
FROM resource_resolvers_v1($1, $2) AS rr
INNER JOIN func_binding_return_value_belongs_to_func_binding_v1($1, $2) AS fbrvbtfb
    ON fbrvbtfb.belongs_to_id = rr.func_binding_id
INNER JOIN func_binding_return_values_v1($1, $2) AS fbrv
    ON fbrv.id = fbrvbtfb.object_id
WHERE
    rr.component_id = $3
ORDER BY rr.id DESC,
         fbrv.id DESC
