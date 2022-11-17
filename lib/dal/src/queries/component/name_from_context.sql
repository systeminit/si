SELECT fbrv.value AS component_name
FROM func_binding_return_values_v1($1, $2) AS fbrv
WHERE id IN (
    SELECT DISTINCT ON (av.attribute_context_prop_id)
        av.func_binding_return_value_id
    FROM attribute_values_v1($1, $2) AS av
    JOIN (
        SELECT name_prop.id
        FROM props_v1($1, $2) AS name_prop
        JOIN prop_belongs_to_prop_v1($1, $2) AS pbtp
            ON name_prop.name = 'name'
                AND pbtp.object_id = name_prop.id
                AND pbtp.belongs_to_id IN (
                    SELECT si_prop.id
                    FROM props_v1($1, $2) AS si_prop
                    JOIN prop_belongs_to_prop_v1($1, $2) AS pbtp
                        ON si_prop.name = 'si'
                            AND pbtp.object_id = si_prop.id
                            AND pbtp.belongs_to_id IN (
                                SELECT pmtmsv.left_object_id AS root_prop_id
                                FROM prop_many_to_many_schema_variants_v1($1, $2) AS pmtmsv
                                JOIN component_belongs_to_schema_variant_v1($1, $2) AS cbtsv
                                    ON cbtsv.belongs_to_id = pmtmsv.right_object_id
                                        AND cbtsv.object_id = $3
                            )
                )
    ) AS name_prop
        ON av.attribute_context_prop_id = name_prop.id
    WHERE in_attribute_context_v1(
        attribute_context_build_from_parts_v1(
            name_prop.id, -- PropId
            -1, -- InternalProviderId
            -1, -- ExternalProviderId
            NULL, -- SchemaId (handled by ComponentId)
            NULL, -- SchemaVariantId (handled by ComponentId)
            $3 -- ComponentId
        ),
        av
    )
    ORDER BY
        av.attribute_context_prop_id,
        av.attribute_context_schema_id DESC,
        av.attribute_context_schema_variant_id DESC,
        av.attribute_context_component_id DESC,
        av.tenancy_universal -- bools sort false first ascending.
)

-- This ends up with an extremely bad query plan to the point where it
-- takes ~10-11 SECONDS to complete, vs the ~12 MILLIseconds that the
-- above version takes.
--
-- SELECT fbrv.value AS component_name
-- FROM component_belongs_to_schema_variant_v1($1, $2) AS cbtsv
-- -- TODO: We could do this as a normal join if we fixed prop_many_to_many_schema_variants to be prop_belongs_to_schema_variant (which matches our current rules/logic)
-- --
-- -- Having the SchemaVariant lets us get the PropId for the "root" Prop.
-- INNER JOIN (
--     SELECT DISTINCT ON (left_object_id)
--         left_object_id AS prop_id,
--         right_object_id AS schema_variant_id
--     FROM prop_many_to_many_schema_variants_v1($1, $2)
-- ) AS root_pmtmsv
--     ON root_pmtmsv.schema_variant_id = cbtsv.belongs_to_id
-- -- Having the "root" PropId lets us get the "si" PropId.
-- INNER JOIN prop_belongs_to_prop_v1($1, $2) AS si_pbtp
--     ON si_pbtp.belongs_to_id = root_pbtsv.prop_id
-- INNER JOIN props_v1($1, $2) AS si_prop
--     ON si_prop.id = si_pbtp.object_id
--         AND si_prop.name = 'si'
-- -- Having the "si" PropId lets us get the "name" PropId.
-- INNER JOIN prop_belongs_to_prop_v1($1, $2) AS name_pbtp
--     ON name_pbtp.belongs_to_id = si_prop.id
-- INNER JOIN props_v1($1, $2) AS name_prop
--     ON name_prop.id = name_pbtp.object_id
--         AND name_prop.name = 'name'
-- INNER JOIN LATERAL (
--     SELECT DISTINCT ON (attribute_context_prop_id)
--         id AS attribute_value_id,
--         func_binding_return_value_id,
--         attribute_context_prop_id,
--         attribute_context_internal_provider_id,
--         attribute_context_external_provider_id,
--         attribute_context_schema_id,
--         attribute_context_schema_variant_id,
--         attribute_context_component_id,
--         attribute_context_system_id
--     FROM attribute_values_v1($1, $2) AS av
--     WHERE
--         in_attribute_context_v1(
--             -- We're only interested in the AttributeValue that's directly for the "/root/si/name" Prop
--             -- for a given ComponentId & SystemId. We're not bothering to filter on the Schema &
--             -- SchemaVariant, as a Component can only belong to one SchemaVariant, and a SchemaVariant
--             -- can only belong to one Schema.
--             attribute_context_build_from_parts_v1(
--                 name_prop.id, -- PropId
--                 -1, -- InternalProviderId
--                 -1, -- ExternalProviderId
--                 NULL, -- SchemaId (handled by ComponentId)
--                 NULL, -- SchemaVariantId (handled by ComponentId)
--                 $3, -- ComponentId
--                 $4 -- SystemId
--             ),
--             av
--         )
--         AND attribute_context_prop_id = name_prop.id
--     ORDER BY
--         attribute_context_prop_id,
--         attribute_context_schema_id DESC,
--         attribute_context_schema_variant_id DESC,
--         attribute_context_component_id DESC,
--         attribute_context_system_id DESC,
--         av.tenancy_universal
-- ) AS name_av
--     ON name_av.attribute_context_prop_id = name_prop.id
-- INNER JOIN func_binding_return_values_v1($1, $2) AS fbrv
--     ON fbrv.id = name_av.func_binding_return_value_id
-- WHERE cbtsv.object_id = $3
