SELECT row_to_json(apa.*) AS object
FROM attribute_prototype_arguments_v1($1, $2) AS apa
         INNER JOIN attribute_prototypes_v1($1, $2) AS ap
                    ON apa.attribute_prototype_id = ap.id
WHERE apa.external_provider_id = $3
  AND ap.attribute_context_internal_provider_id = $4
  AND apa.tail_component_id = $5
  AND apa.head_component_id = $6
