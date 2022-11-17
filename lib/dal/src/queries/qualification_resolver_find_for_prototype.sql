SELECT DISTINCT ON (qualification_resolvers.id) qualification_resolvers.id,
                                                qualification_resolvers.visibility_change_set_pk,
                                                qualification_resolvers.component_id,
                                                qualification_resolvers.schema_id,
                                                qualification_resolvers.schema_variant_id,
                                                row_to_json(qualification_resolvers.*) as object
FROM qualification_resolvers_v1($1, $2) as qualification_resolvers
WHERE qualification_resolvers.qualification_prototype_id = $3
  AND qualification_resolvers.component_id = $4
ORDER BY qualification_resolvers.id,
         visibility_change_set_pk DESC,
         component_id DESC,
         schema_variant_id DESC,
         schema_id DESC;

