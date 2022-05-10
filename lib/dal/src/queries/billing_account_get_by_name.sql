SELECT DISTINCT ON (billing_accounts.id) billing_accounts.id,
                              billing_accounts.visibility_change_set_pk,
                              billing_accounts.visibility_edit_session_pk,
                              row_to_json(billing_accounts.*) AS object
FROM billing_accounts
WHERE billing_accounts.name = $1
  AND in_tenancy_v1($2, billing_accounts.tenancy_universal, billing_accounts.tenancy_billing_account_ids, billing_accounts.tenancy_organization_ids,
                    billing_accounts.tenancy_workspace_ids)
  AND is_visible_v1($3, billing_accounts.visibility_change_set_pk, billing_accounts.visibility_edit_session_pk, billing_accounts.visibility_deleted_at)
ORDER BY id, visibility_change_set_pk DESC, visibility_edit_session_pk DESC
LIMIT 1;
