SELECT DISTINCT ON (confirmation_resolvers.id) confirmation_resolvers.id,
                                                confirmation_resolvers.visibility_change_set_pk,

                                                confirmation_resolvers.component_id,
                                                confirmation_resolvers.schema_id,
                                                confirmation_resolvers.schema_variant_id,
                                                row_to_json(confirmation_resolvers.*) as object
FROM confirmation_resolvers_v1($1, $2) as confirmation_resolvers
WHERE confirmation_resolvers.confirmation_prototype_id = $3
  AND confirmation_resolvers.component_id = $4
  AND confirmation_resolvers.schema_id = $5
  AND confirmation_resolvers.schema_variant_id = $6
ORDER BY confirmation_resolvers.id,
         visibility_change_set_pk DESC,
         component_id DESC,
         schema_variant_id DESC,
         schema_id DESC;

