SELECT DISTINCT ON (
        parent_attribute_value_id,
        attribute_context_prop_id,
        attribute_context_internal_provider_id,
        attribute_context_external_provider_id,
        COALESCE(key, '')
    ) row_to_json(av.*) AS object
FROM attribute_values_v1($1, $2) AS av
    LEFT JOIN (
        SELECT DISTINCT ON (object_id) object_id AS attribute_value_id,
            belongs_to_id AS parent_attribute_value_id
        FROM attribute_value_belongs_to_attribute_value
        WHERE in_tenancy_and_visible_v1(
                $1,
                $2,
                attribute_value_belongs_to_attribute_value
            )
        ORDER BY object_id,
            visibility_change_set_pk DESC,
            visibility_deleted_at DESC NULLS FIRST
    ) AS avbtav ON avbtav.attribute_value_id = av.id
WHERE in_attribute_context_v1($3, av)
ORDER BY parent_attribute_value_id,
    attribute_context_prop_id DESC,
    attribute_context_internal_provider_id DESC,
    attribute_context_external_provider_id DESC,
    COALESCE(key, ''),
    attribute_context_schema_id DESC,
    attribute_context_schema_variant_id DESC,
    attribute_context_component_id DESC,
    attribute_context_system_id DESC;
