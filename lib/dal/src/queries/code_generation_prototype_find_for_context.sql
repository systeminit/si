SELECT DISTINCT ON (code_generation_prototypes.id) code_generation_prototypes.id,
                                                   code_generation_prototypes.component_id,
                                                   code_generation_prototypes.schema_id,
                                                   code_generation_prototypes.schema_variant_id,
                                                   code_generation_prototypes.system_id,
                                                   code_generation_prototypes.visibility_change_set_pk,

                                                   row_to_json(code_generation_prototypes.*) AS object
FROM code_generation_prototypes
WHERE in_tenancy_v1($1, code_generation_prototypes.tenancy_universal,
                    code_generation_prototypes.tenancy_billing_account_ids,
                    code_generation_prototypes.tenancy_organization_ids,
                    code_generation_prototypes.tenancy_workspace_ids)
  AND is_visible_v1($2, code_generation_prototypes.visibility_change_set_pk,
                    code_generation_prototypes.visibility_deleted_at)
  AND (code_generation_prototypes.schema_id = $6
    OR code_generation_prototypes.schema_variant_id = $5
    OR code_generation_prototypes.component_id = $3)
  AND (code_generation_prototypes.system_id = $4 OR code_generation_prototypes.system_id = -1)
ORDER BY code_generation_prototypes.id,
         visibility_change_set_pk DESC,
         component_id DESC,
         func_id DESC,
         system_id DESC,
         schema_variant_id DESC,
         schema_id DESC;
