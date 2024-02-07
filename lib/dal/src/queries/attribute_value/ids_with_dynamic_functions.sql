SELECT avbtap.id AS attribute_value_id
FROM attribute_value_belongs_to_attribute_prototype_v1($1, $2) AS avbtap
    INNER JOIN attribute_prototypes_v1($1, $2) AS ap
        ON ap.id = avbtap.belongs_to_id
    INNER JOIN funcs_v1($1, $2) AS funcs
        ON funcs.id = ap.func_id
WHERE avbtap.object_id = ANY($3)
    AND NOT (
        funcs.name LIKE 'si:set%'
        OR funcs.name = 'si:unset'
    );
