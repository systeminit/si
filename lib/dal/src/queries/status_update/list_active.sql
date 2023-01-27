SELECT row_to_json(status_updates.*) AS object
FROM status_updates
WHERE change_set_pk = $1
 AND finished_at IS NULL
 AND in_tenancy_v1(
     $2,
     status_updates.tenancy_billing_account_pks,
     status_updates.tenancy_organization_pks,
     status_updates.tenancy_workspace_pks
 )
ORDER BY created_at;
