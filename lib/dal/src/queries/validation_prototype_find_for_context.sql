SELECT DISTINCT ON (validation_prototypes.id) validation_prototypes.id,
                                              validation_prototypes.prop_id,
                                              validation_prototypes.schema_id,
                                              validation_prototypes.schema_variant_id,
                                              validation_prototypes.system_id,
                                              validation_prototypes.visibility_change_set_pk,

                                              row_to_json(validation_prototypes.*) AS object
FROM validation_prototypes
WHERE in_tenancy_v1($1, validation_prototypes.tenancy_universal, validation_prototypes.tenancy_billing_account_ids,
                    validation_prototypes.tenancy_organization_ids,
                    validation_prototypes.tenancy_workspace_ids)
  AND is_visible_v1($2, validation_prototypes.visibility_change_set_pk, validation_prototypes.visibility_deleted_at)
  AND validation_prototypes.prop_id = $3
  AND (validation_prototypes.system_id = $4 OR validation_prototypes.system_id = -1)
ORDER BY validation_prototypes.id,
         visibility_change_set_pk DESC,
         prop_id DESC,
         func_id DESC,
         system_id DESC,
         schema_variant_id DESC,
         schema_id DESC;
