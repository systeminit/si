SELECT row_to_json(edit_sessions) AS object
FROM edit_sessions
WHERE edit_sessions.pk = $2
  AND in_tenancy_v1($1, edit_sessions.tenancy_universal, edit_sessions.tenancy_billing_account_ids, edit_sessions.tenancy_organization_ids,
                    edit_sessions.tenancy_workspace_ids);
