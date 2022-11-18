SELECT row_to_json(qualification_resolvers.*) as object
FROM qualification_resolvers_v1($1, $2) AS qualification_resolvers
WHERE
    qualification_resolvers.qualification_prototype_id = $3
    AND qualification_resolvers.component_id = $4
ORDER BY
    component_id DESC,
    schema_variant_id DESC,
    schema_id DESC;

