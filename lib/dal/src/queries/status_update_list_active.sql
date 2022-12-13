SELECT row_to_json(status_updates.*) AS object
FROM status_updates
WHERE change_set_pk = $1
 AND finished_at IS NULL
 AND in_tenancy_v1(
     $2,
     status_updates.tenancy_universal,
     status_updates.tenancy_billing_account_ids,
     status_updates.tenancy_organization_ids,
     status_updates.tenancy_workspace_ids
 )
ORDER BY created_at;
