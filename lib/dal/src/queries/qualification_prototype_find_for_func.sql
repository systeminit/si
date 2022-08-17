SELECT DISTINCT ON (qualification_prototypes.id) qualification_prototypes.id,
                                                 qualification_prototypes.component_id,
                                                 qualification_prototypes.schema_id,
                                                 qualification_prototypes.schema_variant_id,
                                                 qualification_prototypes.system_id,
                                                 qualification_prototypes.visibility_change_set_pk,
                                                 row_to_json(qualification_prototypes.*) AS object
FROM qualification_prototypes
WHERE in_tenancy_v1($1, qualification_prototypes.tenancy_universal,
                    qualification_prototypes.tenancy_billing_account_ids,
                    qualification_prototypes.tenancy_organization_ids,
                    qualification_prototypes.tenancy_workspace_ids)
  AND is_visible_v1($2, qualification_prototypes.visibility_change_set_pk,
                    qualification_prototypes.visibility_deleted_at)
  AND func_id = $3
ORDER BY qualification_prototypes.id,
         visibility_change_set_pk DESC,
         component_id DESC,
         func_id DESC,
         system_id DESC,
         schema_variant_id DESC,
         schema_id DESC;
