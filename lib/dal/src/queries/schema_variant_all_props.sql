SELECT DISTINCT ON (props.id) props.id,
                              props.visibility_change_set_pk,

                              props.visibility_deleted_at,
                              row_to_json(props.*) AS object
FROM props
WHERE in_tenancy_v1(
        $1,
        props.tenancy_universal,
        props.tenancy_billing_account_ids,
        props.tenancy_organization_ids,
        props.tenancy_workspace_ids
    )
  AND is_visible_v1(
        $2,
        props.visibility_change_set_pk,
        props.visibility_deleted_at
    )
  AND props.id IN (WITH RECURSIVE recursive_props AS (SELECT left_object_id AS prop_id
                                                      FROM prop_many_to_many_schema_variants
                                                      WHERE right_object_id = $3
                                                      UNION ALL
                                                      SELECT pbp.object_id AS prop_id
                                                      FROM prop_belongs_to_prop AS pbp
                                                               JOIN recursive_props ON pbp.belongs_to_id = recursive_props.prop_id)
                   SELECT prop_id
                   FROM recursive_props)
ORDER BY id,
         visibility_change_set_pk DESC,
         visibility_deleted_at DESC NULLS FIRST;


