SELECT DISTINCT ON (resource_prototypes.id) resource_prototypes.id,
    resource_prototypes.component_id,
    resource_prototypes.schema_id,
    resource_prototypes.schema_variant_id,
    resource_prototypes.system_id,
    resource_prototypes.visibility_change_set_pk,
    resource_prototypes.visibility_edit_session_pk,
    row_to_json(resource_prototypes.*) AS object
  FROM resource_prototypes
  WHERE in_tenancy_v1($1, resource_prototypes.tenancy_universal, resource_prototypes.tenancy_billing_account_ids, resource_prototypes.tenancy_organization_ids,
      resource_prototypes.tenancy_workspace_ids)
    AND is_visible_v1($2, resource_prototypes.visibility_change_set_pk, resource_prototypes.visibility_edit_session_pk, resource_prototypes.visibility_deleted_at)
    AND (resource_prototypes.schema_id = $6
            OR resource_prototypes.schema_variant_id = $5
            OR resource_prototypes.component_id = $3)
    AND (resource_prototypes.system_id = $4 OR resource_prototypes.system_id = -1)
ORDER BY resource_prototypes.id,
    visibility_change_set_pk DESC,
    visibility_edit_session_pk DESC,
    component_id DESC,
    func_id DESC,
    system_id DESC,
    schema_variant_id DESC,
    schema_id DESC
LIMIT 1;
