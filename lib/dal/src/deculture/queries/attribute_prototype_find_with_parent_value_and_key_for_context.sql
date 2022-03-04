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
INNER JOIN attribute_value_belongs_to_attribute_prototype ON
  attribute_value_belongs_to_attribute_prototype.belongs_to_id = attribute_prototypes.id
INNER JOIN attribute_values ON
  attribute_values.id = attribute_value_belongs_to_attribute_prototype.object_id
  AND in_tenancy_v1($1, attribute_values.tenancy_universal, attribute_values.tenancy_billing_account_ids, attribute_values.tenancy_organization_ids,
                    attribute_values.tenancy_workspace_ids)
  AND is_visible_v1($2, attribute_values.visibility_change_set_pk, attribute_values.visibility_edit_session_pk, attribute_values.visibility_deleted)
LEFT JOIN attribute_value_belongs_to_attribute_value ON
  attribute_value_belongs_to_attribute_value.object_id = attribute_values.id
LEFT JOIN attribute_values AS parent_attribute_values ON
  parent_attribute_values.id = attribute_value_belongs_to_attribute_value.belongs_to_id
  AND in_tenancy_v1($1, parent_attribute_values.tenancy_universal, parent_attribute_values.tenancy_billing_account_ids, parent_attribute_values.tenancy_organization_ids,
                    parent_attribute_values.tenancy_workspace_ids)
  AND is_visible_v1($2, parent_attribute_values.visibility_change_set_pk, parent_attribute_values.visibility_edit_session_pk, parent_attribute_values.visibility_deleted)
WHERE in_tenancy_v1($1, attribute_prototypes.tenancy_universal, attribute_prototypes.tenancy_billing_account_ids, attribute_prototypes.tenancy_organization_ids,
                    attribute_prototypes.tenancy_workspace_ids)
  AND is_visible_v1($2, attribute_prototypes.visibility_change_set_pk, attribute_prototypes.visibility_edit_session_pk, attribute_prototypes.visibility_deleted)
  AND CASE
    WHEN $3::bigint IS NULL THEN parent_attribute_values.id IS NULL
    ELSE parent_attribute_values.id = $3::bigint
  END
  AND CASE
    WHEN $4::text IS NULL THEN attribute_prototypes.key IS NULL
    ELSE attribute_prototypes.key = $4::text
  END
  AND attribute_prototypes.prop_id = $5
  AND attribute_prototypes.schema_id = $6
  AND attribute_prototypes.schema_variant_id = $7
  AND attribute_prototypes.component_id = $8
  AND attribute_prototypes.system_id = $9
ORDER BY prop_id,
         key,
         visibility_change_set_pk DESC,
         visibility_edit_session_pk DESC,
         system_id DESC,
         component_id DESC,
         schema_variant_id DESC,
         schema_id DESC;
