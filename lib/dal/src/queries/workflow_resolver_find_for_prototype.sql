SELECT DISTINCT ON (workflow_resolvers.id) workflow_resolvers.id,
                                                workflow_resolvers.visibility_change_set_pk,

                                                workflow_resolvers.component_id,
                                                workflow_resolvers.schema_id,
                                                workflow_resolvers.schema_variant_id,
                                                workflow_resolvers.system_id,
                                                row_to_json(workflow_resolvers.*) as object
FROM workflow_resolvers
WHERE in_tenancy_v1($1, workflow_resolvers.tenancy_universal, workflow_resolvers.tenancy_billing_account_ids,
                    workflow_resolvers.tenancy_organization_ids,
                    workflow_resolvers.tenancy_workspace_ids)
  AND is_visible_v1($2, workflow_resolvers.visibility_change_set_pk, workflow_resolvers.visibility_deleted_at)
  AND workflow_resolvers.workflow_prototype_id = $3
  AND (workflow_resolvers.component_id = $4
       OR workflow_resolvers.schema_id = $5
       OR workflow_resolvers.schema_variant_id = $6
       OR workflow_resolvers.system_id = $7)
ORDER BY workflow_resolvers.id,
         visibility_change_set_pk DESC,
         component_id DESC,
         system_id DESC,
         schema_variant_id DESC,
         schema_id DESC;

