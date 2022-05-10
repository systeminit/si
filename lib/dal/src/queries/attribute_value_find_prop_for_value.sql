SELECT DISTINCT ON (props.id) props.id,
                              props.visibility_change_set_pk,
                              props.visibility_edit_session_pk,
                              row_to_json(props.*) AS object
FROM props
INNER JOIN attribute_values ON
    attribute_values.attribute_context_prop_id = props.id
    AND in_tenancy_v1($1, attribute_values.tenancy_universal,
                          attribute_values.tenancy_billing_account_ids,
                          attribute_values.tenancy_organization_ids,
                          attribute_values.tenancy_workspace_ids)
    AND is_visible_v1($2, attribute_values.visibility_change_set_pk,
                          attribute_values.visibility_edit_session_pk,
                          attribute_values.visibility_deleted_at)
WHERE in_tenancy_v1($1, props.tenancy_universal, props.tenancy_billing_account_ids,
                        props.tenancy_organization_ids, props.tenancy_workspace_ids)
    AND is_visible_v1($2, props.visibility_change_set_pk, props.visibility_edit_session_pk,
                          props.visibility_deleted_at)
    AND attribute_values.id = $3
ORDER BY props.id,
         visibility_change_set_pk DESC,
         visibility_edit_session_pk DESC
