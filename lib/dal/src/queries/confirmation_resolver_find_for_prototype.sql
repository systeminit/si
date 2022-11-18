SELECT row_to_json(confirmation_resolvers.*) as object
FROM confirmation_resolvers_v1($1, $2) AS confirmation_resolvers
WHERE
    confirmation_resolvers.confirmation_prototype_id = $3
    AND confirmation_resolvers.component_id = $4
    AND confirmation_resolvers.schema_id = $5
    AND confirmation_resolvers.schema_variant_id = $6
