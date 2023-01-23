SELECT DISTINCT ON (av.attribute_context_prop_id) row_to_json(av.*) AS object
FROM attribute_values_v1($1, $2) AS av
         LEFT JOIN attribute_value_belongs_to_attribute_value_v1($1, $2) AS avbtav
                   ON avbtav.object_id = av.id
WHERE in_attribute_context_v1($3, av)
  AND CASE
          WHEN $4::ident IS NULL THEN avbtav.belongs_to_id IS NULL
          ELSE avbtav.belongs_to_id = $4::ident
    END
  AND CASE
          WHEN $5::text IS NULL THEN av.key IS NULL
          ELSE av.key = $5::text
    END
ORDER BY attribute_context_prop_id,
         attribute_context_internal_provider_id DESC,
         attribute_context_external_provider_id DESC,
         attribute_context_component_id DESC
