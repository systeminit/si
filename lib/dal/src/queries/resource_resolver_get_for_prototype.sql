SELECT DISTINCT ON (resource_resolvers.id) resource_resolvers.id,
                                             resource_resolvers.visibility_change_set_pk,
                                             resource_resolvers.visibility_edit_session_pk,
                                             resource_resolvers.component_id,
                                             resource_resolvers.schema_id,
                                             resource_resolvers.schema_variant_id,
                                             resource_resolvers.system_id,
                                             row_to_json(resource_resolvers.*) as object
FROM resource_resolvers
WHERE in_tenancy_v1($1, resource_resolvers.tenancy_universal, resource_resolvers.tenancy_billing_account_ids, resource_resolvers.tenancy_organization_ids,
                    resource_resolvers.tenancy_workspace_ids)
  AND is_visible_v1($2, resource_resolvers.visibility_change_set_pk, resource_resolvers.visibility_edit_session_pk, resource_resolvers.visibility_deleted)
  AND resource_resolvers.resource_prototype_id = $3
  AND resource_resolvers.component_id = $4
ORDER BY resource_resolvers.id,
         visibility_change_set_pk DESC,
         visibility_edit_session_pk DESC,
         component_id DESC,
         system_id DESC,
         schema_variant_id DESC,
         schema_id DESC
LIMIT 1;

