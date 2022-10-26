SELECT
    cgp.format          AS format,
    row_to_json(fbrv.*) AS object
FROM code_generation_resolvers_v1($1, $2) AS cgr
INNER JOIN func_binding_return_value_belongs_to_func_binding_v1($1, $2) AS fbrvbtfb
    ON fbrvbtfb.belongs_to_id = cgr.func_binding_id
INNER JOIN func_binding_return_values_v1($1, $2) AS fbrv
    ON fbrv.id = fbrvbtfb.object_id
INNER JOIN code_generation_prototypes_v1($1, $2) AS cgp
    ON cgp.id = cgr.code_generation_prototype_id
WHERE
    cgr.component_id = $3
    AND (cgr.system_id = $4 OR cgr.system_id = -1);
