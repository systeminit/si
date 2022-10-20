SELECT DISTINCT ON (confirmation_prototypes.func_id) confirmation_prototypes.id,
                                                 confirmation_prototypes.component_id,
                                                 confirmation_prototypes.schema_id,
                                                 confirmation_prototypes.schema_variant_id,
                                                 confirmation_prototypes.system_id,
                                                 confirmation_prototypes.visibility_change_set_pk,

                                                 row_to_json(confirmation_prototypes.*) AS object
FROM confirmation_prototypes
WHERE in_tenancy_v1($1, confirmation_prototypes.tenancy_universal,
                    confirmation_prototypes.tenancy_billing_account_ids,
                    confirmation_prototypes.tenancy_organization_ids,
                    confirmation_prototypes.tenancy_workspace_ids)
  AND is_visible_v1($2, confirmation_prototypes.visibility_change_set_pk,
                    confirmation_prototypes.visibility_deleted_at)
  AND (confirmation_prototypes.schema_id = $6
    OR confirmation_prototypes.schema_variant_id = $5
    OR confirmation_prototypes.component_id = $3)
  AND (confirmation_prototypes.system_id = $4 OR confirmation_prototypes.system_id = -1)
ORDER BY confirmation_prototypes.func_id,
         visibility_change_set_pk DESC,
         component_id DESC,
         func_id DESC,
         system_id DESC,
         schema_variant_id DESC,
         schema_id DESC;
