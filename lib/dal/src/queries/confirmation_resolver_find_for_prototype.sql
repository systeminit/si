SELECT DISTINCT ON (confirmation_resolvers.id) confirmation_resolvers.id,
                                                confirmation_resolvers.visibility_change_set_pk,

                                                confirmation_resolvers.component_id,
                                                confirmation_resolvers.schema_id,
                                                confirmation_resolvers.schema_variant_id,
                                                confirmation_resolvers.system_id,
                                                row_to_json(confirmation_resolvers.*) as object
FROM confirmation_resolvers
WHERE in_tenancy_v1($1, confirmation_resolvers.tenancy_universal, confirmation_resolvers.tenancy_billing_account_ids,
                    confirmation_resolvers.tenancy_organization_ids,
                    confirmation_resolvers.tenancy_workspace_ids)
  AND is_visible_v1($2, confirmation_resolvers.visibility_change_set_pk, confirmation_resolvers.visibility_deleted_at)
  AND confirmation_resolvers.confirmation_prototype_id = $3
  AND confirmation_resolvers.component_id = $4
  AND confirmation_resolvers.schema_id = $5
  AND confirmation_resolvers.schema_variant_id = $6
  AND confirmation_resolvers.system_id = $7
ORDER BY confirmation_resolvers.id,
         visibility_change_set_pk DESC,
         component_id DESC,
         system_id DESC,
         schema_variant_id DESC,
         schema_id DESC;

