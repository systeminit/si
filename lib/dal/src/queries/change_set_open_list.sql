SELECT change_sets.name AS name, change_sets.pk AS value
FROM change_sets
WHERE status = 'Open'
  AND in_tenancy_v1($1, change_sets.tenancy_universal, change_sets.tenancy_billing_account_ids,
                    change_sets.tenancy_organization_ids,
                    change_sets.tenancy_workspace_ids);