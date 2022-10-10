SELECT DISTINCT ON (action_prototypes.id) action_prototypes.id,
                                                 action_prototypes.component_id,
                                                 action_prototypes.schema_id,
                                                 action_prototypes.schema_variant_id,
                                                 action_prototypes.system_id,
                                                 action_prototypes.visibility_change_set_pk,

                                                 row_to_json(action_prototypes.*) AS object
FROM action_prototypes
WHERE in_tenancy_v1($1, action_prototypes.tenancy_universal,
                    action_prototypes.tenancy_billing_account_ids,
                    action_prototypes.tenancy_organization_ids,
                    action_prototypes.tenancy_workspace_ids)
  AND is_visible_v1($2, action_prototypes.visibility_change_set_pk,
                    action_prototypes.visibility_deleted_at)
  AND action_prototypes.name = $3
  AND action_prototypes.schema_id = $6
  AND action_prototypes.schema_variant_id = $5
  AND (action_prototypes.system_id = $4 OR action_prototypes.system_id = -1)
ORDER BY action_prototypes.id,
         visibility_change_set_pk DESC,
         component_id DESC,
         system_id DESC,
         schema_variant_id DESC,
         schema_id DESC;
