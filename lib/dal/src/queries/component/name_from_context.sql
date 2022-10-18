SELECT DISTINCT ON (id) value AS component_name
FROM func_binding_return_values AS fbrv
INNER JOIN (
    SELECT DISTINCT ON (attribute_context_prop_id)
        id AS attribute_value_id,
        func_binding_return_value_id,
        attribute_context_prop_id,
        attribute_context_internal_provider_id,
        attribute_context_external_provider_id,
        attribute_context_schema_id,
        attribute_context_schema_variant_id,
        attribute_context_component_id,
        attribute_context_system_id
    FROM attribute_values AS av
    INNER JOIN (
        -- Having the "si" PropId lets us get the "name" PropId.
        SELECT DISTINCT ON (object_id) object_id AS name_prop_id
        FROM prop_belongs_to_prop AS pbtp
        INNER JOIN (
            -- Having the "root" PropId lets us get the "si" PropId.
            SELECT DISTINCT ON (object_id) object_id AS si_prop_id
            FROM prop_belongs_to_prop AS pbtp
            INNER JOIN (
                SELECT DISTINCT ON (id) id AS prop_id
                FROM props
                WHERE
                    in_tenancy_and_visible_v1($1, $2, props)
                    AND name = 'si'
                ORDER BY
                    id,
                    visibility_change_set_pk DESC,
                    visibility_deleted_at DESC NULLS FIRST
            ) AS si_prop_info ON si_prop_info.prop_id = pbtp.object_id
            INNER JOIN (
                -- Having the SchemaVariant lets us get the PropId for the "root" Prop.
                SELECT DISTINCT ON (right_object_id) left_object_id AS root_prop_id
                FROM prop_many_to_many_schema_variants AS pmtmsv
                INNER JOIN (
                    -- Need to grab the SchemaVariant the Component belongs to so we can get at the root PropId.
                    SELECT DISTINCT ON (object_id) belongs_to_id AS schema_variant_id
                    FROM component_belongs_to_schema_variant AS cbtsv
                    WHERE
                        in_tenancy_and_visible_v1($1, $2, cbtsv)
                        AND object_id = $3
                    ORDER BY
                        object_id,
                        visibility_change_set_pk DESC,
                        visibility_deleted_at DESC NULLS FIRST
                ) AS sv_info ON sv_info.schema_variant_id = pmtmsv.right_object_id
                WHERE in_tenancy_and_visible_v1($1, $2, pmtmsv)
                ORDER BY
                    right_object_id,
                    visibility_change_set_pk DESC,
                    visibility_deleted_at DESC NULLS FIRST
            ) AS root_prop_info ON root_prop_info.root_prop_id = pbtp.belongs_to_id
            WHERE in_tenancy_and_visible_v1($1, $2, pbtp)
            ORDER BY
                object_id,
                visibility_change_set_pk DESC,
                visibility_deleted_at DESC NULLS FIRST
        ) AS si_prop_info ON si_prop_info.si_prop_id = pbtp.belongs_to_id
        INNER JOIN (
            SELECT DISTINCT ON (id) id AS prop_id
            FROM props
            WHERE
                in_tenancy_and_visible_v1($1, $2, props)
                AND name = 'name'
            ORDER BY
                id,
                visibility_change_set_pk DESC,
                visibility_deleted_at DESC NULLS FIRST
        ) AS name_prop ON name_prop.prop_id = pbtp.object_id
        ORDER BY
            object_id,
            visibility_change_set_pk DESC,
            visibility_deleted_at DESC
    ) AS name_prop_info ON name_prop_info.name_prop_id = av.attribute_context_prop_id
    WHERE
        in_tenancy_and_visible_v1($1, $2, av)
        AND in_attribute_context_v1(
            -- We're only interested in the AttributeValue that's directly for the "/root/si/name" Prop
            -- for a given ComponentId & SystemId. We're not bothering to filter on the Schema &
            -- SchemaVariant, as a Component can only belong to one SchemaVariant, and a SchemaVariant
            -- can only belong to one Schema.
            attribute_context_build_from_parts_v1(
                name_prop_info.name_prop_id, -- PropId
                -1, -- InternalProviderId
                -1, -- ExternalProviderId
                NULL, -- SchemaId (handled by ComponentId)
                NULL, -- SchemaVariantId (handled by ComponentId)
                $3, -- ComponentId
                $4 -- SystemId
            ),
            av
        )
    ORDER BY
        attribute_context_prop_id,
        visibility_change_set_pk DESC,
        visibility_deleted_at DESC NULLS FIRST,
        attribute_context_schema_id DESC,
        attribute_context_schema_variant_id DESC,
        attribute_context_component_id DESC,
        attribute_context_system_id DESC
) AS av_info ON av_info.func_binding_return_value_id = fbrv.id
WHERE in_tenancy_and_visible_v1($1, $2, fbrv)
ORDER BY
    id,
    visibility_change_set_pk DESC,
    visibility_deleted_at DESC NULLS FIRST;
