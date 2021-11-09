SELECT true AS authorized, users.id, users.visibility_change_set_pk, users.visibility_edit_session_pk
FROM users
         INNER JOIN group_many_to_many_users
                    ON users.id = group_many_to_many_users.right_object_id
                        AND users.visibility_edit_session_pk = group_many_to_many_users.visibility_edit_session_pk
                        AND users.visibility_change_set_pk = group_many_to_many_users.visibility_change_set_pk
                        AND users.visibility_deleted = group_many_to_many_users.visibility_deleted
         INNER JOIN capability_belongs_to_group
                    ON capability_belongs_to_group.belongs_to_id = group_many_to_many_users.left_object_id
                        AND users.visibility_edit_session_pk = capability_belongs_to_group.visibility_edit_session_pk
                        AND users.visibility_change_set_pk = capability_belongs_to_group.visibility_change_set_pk
                        AND users.visibility_deleted = capability_belongs_to_group.visibility_deleted
         INNER JOIN capabilities
             ON capabilities.id = capability_belongs_to_group.object_id
                 AND users.visibility_edit_session_pk = capabilities.visibility_edit_session_pk
                 AND users.visibility_change_set_pk = capabilities.visibility_change_set_pk
                 AND users.visibility_deleted = capabilities.visibility_deleted
                 AND capabilities.subject = 'any'
                 AND capabilities.action = 'any'
WHERE users.id = $3
  AND in_tenancy_v1($1, users.tenancy_universal, users.tenancy_billing_account_ids, users.tenancy_organization_ids,
                    users.tenancy_workspace_ids)
  AND is_visible_v1($2, users.visibility_change_set_pk, users.visibility_edit_session_pk, users.visibility_deleted)
ORDER BY id, visibility_change_set_pk ASC, visibility_edit_session_pk ASC
LIMIT 1;