SELECT DISTINCT ON (attribute_prototypes.id) attribute_prototypes.id,
                                             row_to_json(attribute_prototypes.*) AS object
FROM attribute_prototypes
WHERE in_tenancy_v1($1, attribute_prototypes.tenancy_universal,
                    attribute_prototypes.tenancy_billing_account_ids,
                    attribute_prototypes.tenancy_organization_ids,
                    attribute_prototypes.tenancy_workspace_ids)
  AND is_visible_v1($2, attribute_prototypes.visibility_change_set_pk,
                    attribute_prototypes.visibility_deleted_at)
  AND func_id = $3
ORDER BY attribute_prototypes.id;