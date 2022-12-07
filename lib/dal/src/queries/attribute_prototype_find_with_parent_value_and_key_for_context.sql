SELECT DISTINCT ON (
    ap.attribute_context_prop_id,
    COALESCE(ap.key, '')
)
    row_to_json(ap.*) AS object
FROM attribute_prototypes_v1($1, $2) AS ap
INNER JOIN attribute_value_belongs_to_attribute_prototype_v1($1, $2) AS avbtap
    ON avbtap.belongs_to_id = ap.id
INNER JOIN attribute_values_v1($1, $2) AS av
    ON av.id = avbtap.object_id
LEFT JOIN attribute_value_belongs_to_attribute_value_v1($1, $2) AS avbtav
    ON aavbtav.object_id = av.id
LEFT JOIN attribute_values_v1($1, $2) AS parent_attribute_values
    ON parent_attribute_values.id = avbtav.belongs_to_id
WHERE
    exact_attribute_context_v1($3, ap)
    AND CASE
            WHEN $4::ident IS NULL THEN parent_attribute_values.id IS NULL
            ELSE parent_attribute_values.id = $4::ident
        END
    AND CASE
            WHEN $5::text IS NULL THEN ap.key IS NULL
            ELSE ap.key = $5::text
        END
ORDER BY
    attribute_context_prop_id,
    COALESCE(key, ''),
    visibility_change_set_pk DESC,
    attribute_context_internal_provider_id DESC,
    attribute_context_external_provider_id DESC,
    attribute_context_component_id DESC,
    av.tenancy_universal -- bools sort false first ascending.
