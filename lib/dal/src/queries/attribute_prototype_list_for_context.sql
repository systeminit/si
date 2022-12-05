SELECT DISTINCT ON (
    attribute_context_prop_id,
    COALESCE(key, '')
    ) row_to_json(ap.*) AS object
FROM attribute_prototypes_v1($1, $2) AS ap
WHERE in_attribute_context_v1($3, ap)
  AND attribute_context_prop_id = $4
ORDER BY attribute_context_prop_id,
         COALESCE(key, ''),
         attribute_context_internal_provider_id DESC,
         attribute_context_external_provider_id DESC,
         attribute_context_component_id DESC,
         ap.tenancy_universal -- bools sort false first ascending.
