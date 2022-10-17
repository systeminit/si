SELECT
    parent_prop_id IS NULL AS is_for_root_prop
FROM (
    SELECT DISTINCT ON (id) prop_id
    FROM internal_providers
    WHERE
        in_tenancy_and_visible_v1($1, $2, internal_providers)
        AND id = ($3::jsonb ->> 'attribute_context_internal_provider_id')::bigint
    ORDER BY
        id,
        visibility_change_set_pk DESC,
        visibility_deleted_at DESC NULLS FIRST
) AS ip
LEFT JOIN (
    SELECT DISTINCT ON (object_id)
        object_id AS child_prop_id,
        belongs_to_id AS parent_prop_id
    FROM prop_belongs_to_prop
    WHERE in_tenancy_and_visible_v1($1, $2, prop_belongs_to_prop)
    ORDER BY
        object_id,
        visibility_change_set_pk DESC,
        visibility_deleted_at DESC NULLS FIRST
) AS prop_info ON prop_info.child_prop_id = ip.prop_id
