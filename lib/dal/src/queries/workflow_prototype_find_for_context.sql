SELECT DISTINCT ON (workflow_prototypes.id) workflow_prototypes.id,
                                                 workflow_prototypes.component_id,
                                                 workflow_prototypes.schema_id,
                                                 workflow_prototypes.schema_variant_id,
                                                 workflow_prototypes.system_id,
                                                 workflow_prototypes.visibility_change_set_pk,

                                                 row_to_json(workflow_prototypes.*) AS object
FROM workflow_prototypes
WHERE in_tenancy_v1($1, workflow_prototypes.tenancy_universal,
                    workflow_prototypes.tenancy_billing_account_ids,
                    workflow_prototypes.tenancy_organization_ids,
                    workflow_prototypes.tenancy_workspace_ids)
  AND is_visible_v1($2, workflow_prototypes.visibility_change_set_pk,
                    workflow_prototypes.visibility_deleted_at)
  AND workflow_prototypes.schema_id = $6
  AND workflow_prototypes.schema_variant_id = $5
  AND workflow_prototypes.component_id = $3
  AND (workflow_prototypes.system_id = $4 OR workflow_prototypes.system_id = -1)
ORDER BY workflow_prototypes.id,
         visibility_change_set_pk DESC,
         component_id DESC,
         func_id DESC,
         system_id DESC,
         schema_variant_id DESC,
         schema_id DESC;
