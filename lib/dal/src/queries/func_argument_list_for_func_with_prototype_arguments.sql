SELECT DISTINCT
    ON (func_arguments.name) func_arguments.name,
                             row_to_json(func_arguments.*) as func_argument_object,
                             row_to_json(apa.*)            AS prototype_argument_object
FROM func_arguments
         LEFT JOIN attribute_prototype_arguments apa
                   ON func_arguments.id = apa.func_argument_id
                       AND apa.attribute_prototype_id = $4
                       AND in_tenancy_and_visible_v1($1, $2, apa)
WHERE in_tenancy_and_visible_v1($1, $2, func_arguments)
  AND func_arguments.func_id = $3
ORDER BY func_arguments.name,
         func_arguments.visibility_change_set_pk DESC,
         func_arguments.visibility_deleted_at DESC NULLS FIRST;