SELECT DISTINCT ON (workflow_runners.id) workflow_runners.id,
                                                workflow_runners.visibility_change_set_pk,

                                                workflow_runners.component_id,
                                                workflow_runners.schema_id,
                                                workflow_runners.schema_variant_id,
                                                workflow_runners.system_id,
                                                row_to_json(workflow_runners.*) as object
FROM workflow_runners
WHERE in_tenancy_v1($1, workflow_runners.tenancy_universal, workflow_runners.tenancy_billing_account_ids,
                    workflow_runners.tenancy_organization_ids,
                    workflow_runners.tenancy_workspace_ids)
  AND is_visible_v1($2, workflow_runners.visibility_change_set_pk, workflow_runners.visibility_deleted_at)
  AND workflow_runners.workflow_prototype_id = $3
  AND (workflow_runners.component_id = $4
       OR workflow_runners.schema_id = $5
       OR workflow_runners.schema_variant_id = $6
       OR workflow_runners.system_id = $7)
ORDER BY workflow_runners.id,
         visibility_change_set_pk DESC,
         component_id DESC,
         system_id DESC,
         schema_variant_id DESC,
         schema_id DESC;

