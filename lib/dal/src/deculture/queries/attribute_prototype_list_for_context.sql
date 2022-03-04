SELECT DISTINCT ON (attribute_prototypes.prop_id, attribute_prototypes.key) attribute_prototypes.id,
                              attribute_prototypes.prop_id,
                              attribute_prototypes.key,
                              attribute_prototypes.visibility_change_set_pk,
                              attribute_prototypes.visibility_edit_session_pk,
                              attribute_prototypes.component_id,
                              attribute_prototypes.schema_id,
                              attribute_prototypes.schema_variant_id,
                              attribute_prototypes.system_id,
                              row_to_json(attribute_prototypes.*) AS object
FROM attribute_prototypes
WHERE in_tenancy_v1($1, attribute_prototypes.tenancy_universal, attribute_prototypes.tenancy_billing_account_ids, attribute_prototypes.tenancy_organization_ids,
                    attribute_prototypes.tenancy_workspace_ids)
  AND is_visible_v1($2, attribute_prototypes.visibility_change_set_pk, attribute_prototypes.visibility_edit_session_pk, attribute_prototypes.visibility_deleted)
  AND attribute_prototypes.prop_id = $3
  AND (attribute_prototypes.component_id = $4 OR attribute_prototypes.component_id = -1)
  AND (attribute_prototypes.schema_id = $5 OR attribute_prototypes.schema_id = -1)
  AND (attribute_prototypes.schema_variant_id = $6 OR attribute_prototypes.schema_variant_id = -1)
  AND (attribute_prototypes.system_id = $7 OR attribute_prototypes.system_id = -1)
ORDER BY prop_id,
         key,
         visibility_change_set_pk DESC,
         visibility_edit_session_pk DESC,
         component_id DESC,
         system_id DESC,
         schema_variant_id DESC,
         schema_id DESC;
