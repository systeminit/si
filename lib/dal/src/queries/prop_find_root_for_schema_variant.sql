SELECT DISTINCT ON (props.id) props.id,
                              props.visibility_change_set_pk,
                              props.visibility_edit_session_pk,
                              props.visibility_deleted_at,
                              row_to_json(props.*) as object

FROM props
         INNER JOIN prop_many_to_many_schema_variants
                    ON prop_many_to_many_schema_variants.left_object_id = props.id

WHERE in_tenancy_v1($1, props.tenancy_universal,
                    props.tenancy_billing_account_ids,
                    props.tenancy_organization_ids,
                    props.tenancy_workspace_ids)
  AND is_visible_v1($2, props.visibility_change_set_pk,
                    props.visibility_edit_session_pk,
                    props.visibility_deleted_at)
  AND prop_many_to_many_schema_variants.right_object_id = $3

ORDER BY props.id,
         props.visibility_change_set_pk DESC,
         props.visibility_edit_session_pk DESC,
         props.visibility_deleted_at DESC NULLS FIRST