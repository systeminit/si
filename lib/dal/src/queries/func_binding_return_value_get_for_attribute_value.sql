SELECT DISTINCT ON (func_binding_return_values.id)
       func_binding_return_values.id,
       func_binding_return_values.visibility_change_set_pk,
       func_binding_return_values.visibility_edit_session_pk,
       func_binding_return_values.visibility_deleted_at,
       row_to_json(func_binding_return_values.*) as object
FROM func_binding_return_values
INNER JOIN attribute_values ON
      func_binding_return_values.id = attribute_values.func_binding_return_value_id
      AND in_tenancy_v1($1, attribute_values.tenancy_universal,
                            attribute_values.tenancy_billing_account_ids,
                            attribute_values.tenancy_organization_ids,
                            attribute_values.tenancy_workspace_ids)
      AND is_visible_v1($2, attribute_values.visibility_change_set_pk,
                            attribute_values.visibility_edit_session_pk,
                            attribute_values.visibility_deleted_at)
WHERE in_tenancy_v1($1, func_binding_return_values.tenancy_universal,
                        func_binding_return_values.tenancy_billing_account_ids,
                        func_binding_return_values.tenancy_organization_ids,
                        func_binding_return_values.tenancy_workspace_ids)
      AND is_visible_v1($2, func_binding_return_values.visibility_change_set_pk,
                            func_binding_return_values.visibility_edit_session_pk,
                            func_binding_return_values.visibility_deleted_at)
      AND attribute_values.id = $3
ORDER BY func_binding_return_values.id
         func_binding_return_values.visibility_change_set_pk DESC,
         func_binding_return_values.visibility_edit_session_pk DESC;
         func_binding_return_values.visibility_deleted_at DESC NULLS FIRST;
