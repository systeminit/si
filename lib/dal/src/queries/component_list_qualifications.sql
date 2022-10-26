SELECT
    row_to_json(qp)     AS prototype,
    row_to_json(fbrv.*) AS object,
    json_build_object(
        'display_name', f.display_name,
        'description',  f.description,
        'link',         f.link
    )                                         AS func_metadata_view
FROM qualification_resolvers_v1($1, $2) AS qr
INNER JOIN func_binding_return_value_belongs_to_func_binding_v1($1, $2) AS fbrvbtfb
    ON fbrvbtfb.belongs_to_id = qr.func_binding_id
INNER JOIN func_binding_return_values_v1($1, $2) AS fbrv
    ON fbrv.id = fbrvbtfb.object_id
INNER JOIN qualification_prototypes_v1($1, $2) AS qp
    ON qp.id = qr.qualification_prototype_id
INNER JOIN funcs_v1($1, $2) AS f
    ON f.id = qp.func_id
WHERE
    qr.component_id = $3
    AND (qr.system_id = $4 OR qr.system_id = -1);
