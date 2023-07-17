SELECT row_to_json(aps.*) AS object
FROM attribute_prototypes_v1($1, $2) AS aps
WHERE
  aps.attribute_context_prop_id IN
    (SELECT id FROM props_v1($1, $2) AS props WHERE props.schema_variant_id = $3)
  OR
  aps.attribute_context_internal_provider_id IN
    (SELECT id FROM internal_providers_v1($1, $2) AS ips WHERE ips.schema_variant_id = $3)
  OR
  aps.attribute_context_external_provider_id IN
    (SELECT id FROM external_providers_v1($1, $2) AS eps WHERE eps.schema_variant_id = $3)
ORDER BY aps.id DESC
