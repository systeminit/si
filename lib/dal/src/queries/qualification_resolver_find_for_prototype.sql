SELECT DISTINCT ON (qualification_resolvers.id) qualification_resolvers.id,
                                                qualification_resolvers.visibility_change_set_pk,

                                                qualification_resolvers.component_id,
                                                qualification_resolvers.schema_id,
                                                qualification_resolvers.schema_variant_id,
                                                qualification_resolvers.system_id,
                                                row_to_json(qualification_resolvers.*) as object
FROM qualification_resolvers
WHERE in_tenancy_v1($1, qualification_resolvers.tenancy_universal, qualification_resolvers.tenancy_billing_account_ids,
                    qualification_resolvers.tenancy_organization_ids,
                    qualification_resolvers.tenancy_workspace_ids)
  AND is_visible_v1($2, qualification_resolvers.visibility_change_set_pk, qualification_resolvers.visibility_deleted_at)
  AND qualification_resolvers.qualification_prototype_id = $3
  AND qualification_resolvers.component_id = $4
ORDER BY qualification_resolvers.id,
         visibility_change_set_pk DESC,
         component_id DESC,
         system_id DESC,
         schema_variant_id DESC,
         schema_id DESC;

