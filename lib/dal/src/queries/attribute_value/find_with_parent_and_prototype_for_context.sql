SELECT DISTINCT ON (av.attribute_context_prop_id) row_to_json(av.*) AS object
FROM attribute_values_v1($1, $2) AS av

-- Scope by attribute prototype. We need these for handling elements in arrays and values in maps.
         INNER JOIN attribute_value_belongs_to_attribute_prototype_v1($1, $2) AS avbtap
                    ON avbtap.object_id = av.id
         INNER JOIN attribute_prototypes_v1($1, $2) AS ap
                    ON ap.id = avbtap.belongs_to_id

-- Handle parentage. We need to use LEFT JOINs here to not wipe out attribute values that do not have relevant parents.
         LEFT JOIN attribute_value_belongs_to_attribute_value_v1($1, $2) AS avbtav
                   ON avbtav.object_id = av.id
         LEFT JOIN attribute_values_v1($1, $2) AS parent_attribute_values
                   ON parent_attribute_values.id = avbtav.belongs_to_id

WHERE exact_attribute_context_v1($3, av)
  AND ap.id = $4
  AND CASE
          WHEN $5::ident IS NULL THEN parent_attribute_values.id IS NULL
          ELSE parent_attribute_values.id = $5::ident
    END
ORDER BY attribute_context_prop_id,
         attribute_context_internal_provider_id DESC,
         attribute_context_external_provider_id DESC,
         attribute_context_component_id DESC
