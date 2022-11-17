/*
    This query groups arguments that belong to an attribute prototype by name. For every argument that shares the same
    name, they will be in the same "array".

    { key: name, value: [argument_with_same_name_1, argument_with_same_name_2] },
    { key: name, value: [argument_that_only_has_this_name] }
 */
SELECT row_to_json(prototype_args) AS object
FROM (
    SELECT
        attribute_prototype_id,
        name AS argument_name,
        array_agg(CASE WHEN internal_provider_data.internal_provider_id IS NOT NULL THEN internal_provider_data.value ELSE external_provider_data.value END) AS values
    FROM (
        SELECT DISTINCT ON (attribute_prototype_arguments.id)
            attribute_prototype_arguments.attribute_prototype_id,
            fa.name,
            attribute_prototype_arguments.internal_provider_id,
            attribute_prototype_arguments.external_provider_id,
            attribute_prototype_arguments.tail_component_id
        FROM attribute_prototype_arguments
        INNER JOIN func_arguments_v1($1, $2) AS fa
            ON attribute_prototype_arguments.func_argument_id = fa.id
        WHERE
            attribute_prototype_arguments.attribute_prototype_id = $3
            AND CASE WHEN attribute_prototype_arguments.external_provider_id != -1
                THEN
                    attribute_prototype_arguments.head_component_id = $4
                ELSE
                    TRUE
            END
        ORDER BY
            attribute_prototype_arguments.id,
            attribute_prototype_arguments.visibility_change_set_pk DESC,
            attribute_prototype_arguments.visibility_deleted_at DESC NULLS FIRST
    ) AS prototype_argument_data
    -- Get the values for InternalProviders
    LEFT JOIN LATERAL (
        SELECT DISTINCT ON (attribute_context_internal_provider_id)
            av.id,
            attribute_context_internal_provider_id AS internal_provider_id,
            fbrv.value,
            attribute_context_prop_id,
            attribute_context_internal_provider_id,
            attribute_context_external_provider_id,
            attribute_context_schema_id,
            attribute_context_schema_variant_id,
            attribute_context_component_id
        FROM attribute_values_v1($1, $2) AS av
        INNER JOIN func_binding_return_values_v1($1, $2) AS fbrv
            ON av.func_binding_return_value_id = fbrv.id
        WHERE
            -- We want to override the Prop/ExternalProvider/InternalProvider information on the AttributeContext
            -- that we're provided to make sure that we're looking for AttributeValues for the particular
            -- InternalProvider that we're interested in at this point. `jsonb || jsonb` is the union of the two,
            -- taking values from the second where there are conflicts.
            --
            -- # SELECT '{"a": "b", "c": "d"}'::jsonb || '{"a": "foo", "e": "f"}'::jsonb;
            --              ?column?
            -- ----------------------------------
            --  {"a": "foo", "c": "d", "e": "f"}
            -- (1 row)
            in_attribute_context_v1(
                $5 || jsonb_build_object(
                    'attribute_context_prop_id',              -1,
                    'attribute_context_external_provider_id', -1,
                    -- The reference to `prototype_argument_data` is why this needs to be a `LATERAL` join.
                    'attribute_context_internal_provider_id', prototype_argument_data.internal_provider_id
                ),
                av
            )
        ORDER BY
            attribute_context_internal_provider_id,
            attribute_context_schema_id DESC,
            attribute_context_schema_variant_id DESC,
            attribute_context_component_id DESC,
            av.tenancy_universal -- bools sort false first ascending.
    ) AS internal_provider_data ON prototype_argument_data.internal_provider_id = internal_provider_data.internal_provider_id
    LEFT JOIN LATERAL (
        SELECT DISTINCT ON (attribute_context_external_provider_id)
            av.id,
            attribute_context_external_provider_id AS external_provider_id,
            value,
            attribute_context_prop_id,
            attribute_context_internal_provider_id,
            attribute_context_external_provider_id,
            attribute_context_schema_id,
            attribute_context_schema_variant_id,
            attribute_context_component_id
        FROM attribute_values_v1($1, $2) AS av
        INNER JOIN func_binding_return_values_v1($1, $2) AS fbrv
            ON av.func_binding_return_value_id = fbrv.id
        WHERE
            -- We're also overriding the AttributeContext's ComponentId, SchemaId, and SchemaVariantId here,
            -- because the source data is coming from a different Component (and potentially
            -- Schema & SchemaVaiant) from where we're trying to set the final value.
            in_attribute_context_v1(
                $5 || jsonb_build_object(
                    'attribute_context_prop_id',              -1,
                    'attribute_context_external_provider_id', prototype_argument_data.external_provider_id,
                    'attribute_context_internal_provider_id', -1,
                    'attribute_context_component_id',         prototype_argument_data.tail_component_id,
                    'attribute_context_schema_id', NULL,
                    'attribute_context_schema_variant_id', NULL
                ),
                av
            )
        ORDER BY
            attribute_context_external_provider_id,
            attribute_context_schema_id DESC,
            attribute_context_schema_variant_id DESC,
            attribute_context_component_id DESC,
            av.tenancy_universal -- bools sort false first ascending.
    ) AS external_provider_data ON prototype_argument_data.external_provider_id = external_provider_data.external_provider_id
    GROUP BY
        attribute_prototype_id,
        name
) AS prototype_args
