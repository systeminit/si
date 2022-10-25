SELECT DISTINCT ON (fix_resolvers.id) fix_resolvers.id,
                                                fix_resolvers.visibility_change_set_pk,

                                                fix_resolvers.component_id,
                                                fix_resolvers.schema_id,
                                                fix_resolvers.schema_variant_id,
                                                fix_resolvers.system_id,
                                                row_to_json(fix_resolvers.*) as object
FROM fix_resolvers
WHERE in_tenancy_v1($1, fix_resolvers.tenancy_universal, fix_resolvers.tenancy_billing_account_ids,
                    fix_resolvers.tenancy_organization_ids,
                    fix_resolvers.tenancy_workspace_ids)
  AND is_visible_v1($2, fix_resolvers.visibility_change_set_pk, fix_resolvers.visibility_deleted_at)
  AND fix_resolvers.confirmation_resolver_id = $3
  AND fix_resolvers.component_id = $4
  AND fix_resolvers.schema_id = $5
  AND fix_resolvers.schema_variant_id = $6
  AND fix_resolvers.system_id = $7
ORDER BY fix_resolvers.id,
         visibility_change_set_pk DESC,
         component_id DESC,
         system_id DESC,
         schema_variant_id DESC,
         schema_id DESC;
