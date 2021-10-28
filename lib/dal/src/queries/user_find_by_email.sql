SELECT DISTINCT ON (users.id) users.id,
                              users.visibility_change_set_pk,
                              users.visibility_edit_session_pk,
                              row_to_json(users.*) AS object
FROM users
WHERE users.email = $1
  AND in_tenancy_v1($2, users.tenancy_universal, users.tenancy_billing_account_ids, users.tenancy_organization_ids,
                    users.tenancy_workspace_ids)
  AND is_visible_v1($3, users.visibility_change_set_pk, users.visibility_edit_session_pk, users.visibility_deleted)
ORDER BY id, visibility_change_set_pk ASC, visibility_edit_session_pk ASC
LIMIT 1;