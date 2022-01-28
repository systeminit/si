SELECT DISTINCT ON (resources.id) resources.id,
    row_to_json(resources.*) AS object
FROM resources
INNER JOIN resource_belongs_to_component
  ON resources.id = resource_belongs_to_component.object_id
    AND resources.tenancy_universal = resource_belongs_to_component.tenancy_universal
    AND resources.tenancy_billing_account_ids = resource_belongs_to_component.tenancy_billing_account_ids
    AND resources.tenancy_organization_ids = resource_belongs_to_component.tenancy_organization_ids
    AND resources.tenancy_workspace_ids = resource_belongs_to_component.tenancy_workspace_ids
INNER JOIN resource_belongs_to_system
  ON resources.id = resource_belongs_to_system.object_id
    AND resources.tenancy_universal = resource_belongs_to_system.tenancy_universal
    AND resources.tenancy_billing_account_ids = resource_belongs_to_system.tenancy_billing_account_ids
    AND resources.tenancy_organization_ids = resource_belongs_to_system.tenancy_organization_ids
    AND resources.tenancy_workspace_ids = resource_belongs_to_system.tenancy_workspace_ids
WHERE in_tenancy_v1($1, resources.tenancy_universal, resources.tenancy_billing_account_ids, resources.tenancy_organization_ids,
    resources.tenancy_workspace_ids)
  AND is_visible_v1($2, resources.visibility_change_set_pk, resources.visibility_edit_session_pk, resources.visibility_deleted)
  AND resource_belongs_to_component.belongs_to_id = $3
  AND resource_belongs_to_system.belongs_to_id = $4
ORDER BY resources.id,
    resources.visibility_change_set_pk DESC,
    resources.visibility_edit_session_pk DESC
LIMIT 1;
