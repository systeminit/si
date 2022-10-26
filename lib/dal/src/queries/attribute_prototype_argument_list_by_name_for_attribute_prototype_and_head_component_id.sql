/*
    This query groups arguments that belong to an attribute prototype by name. For every argument that shares the same
    name, they will be in the same "array".

    { key: name, value: [argument_with_same_name_1, argument_with_same_name_2] },
    { key: name, value: [argument_that_only_has_this_name] }
 */

SELECT name, array_agg(arguments) as arguments
FROM (
    SELECT
        fa.name                                      AS name,
        row_to_json(apa.*) AS arguments
    FROM attribute_prototype_arguments_v1($1, $2) AS apa
    JOIN func_arguments_v1($1, $2) AS fa
        ON apa.func_argument_id = fa.id
    WHERE
        apa.attribute_prototype_id = $3
        AND CASE
                WHEN external_provider_id != -1 THEN
                    head_component_id = $4
                ELSE
                    TRUE
            END
) as apa_found
GROUP BY name;
