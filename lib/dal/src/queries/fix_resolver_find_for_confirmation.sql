SELECT row_to_json(fix_resolvers.*) as object
FROM fix_resolvers_v1($1, $2) as fix_resolvers
WHERE fix_resolvers.confirmation_resolver_id = $3
  AND fix_resolvers.component_id = $4
  AND fix_resolvers.schema_id = $5
  AND fix_resolvers.schema_variant_id = $6
ORDER BY fix_resolvers.id,
         component_id DESC,
         schema_variant_id DESC,
         schema_id DESC;
