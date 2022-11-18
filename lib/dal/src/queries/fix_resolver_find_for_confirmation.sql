SELECT row_to_json(fix_resolvers.*) as object
FROM fix_resolvers_v1($1, $2) AS fix_resolvers
WHERE
    fix_resolvers.confirmation_resolver_id = $3
    AND fix_resolvers.component_id = $4
    AND fix_resolvers.schema_id = $5
    AND fix_resolvers.schema_variant_id = $6
