SELECT DISTINCT ON (attribute_values.attribute_context_prop_id) row_to_json(attribute_values.*) AS object
FROM attribute_values_v1($1, $2) AS attribute_values
JOIN (
  SELECT si_child_prop.id
  FROM props_v1($1, $2) AS si_child_prop
  JOIN prop_belongs_to_prop_v1($1, $2) AS si_child_prop_belongs_to_si_prop
    ON si_child_prop_belongs_to_si_prop.object_id = si_child_prop.id
       AND si_child_prop.name = $5
  JOIN props_v1($1, $2) as si_prop
    ON si_child_prop_belongs_to_si_prop.belongs_to_id = si_prop.id
       AND si_prop.name = 'si'
  JOIN prop_belongs_to_prop_v1($1, $2) AS si_prop_belongs_to_root_prop
    ON si_prop_belongs_to_root_prop.object_id = si_prop.id
  JOIN schema_variants_v1($1, $2) AS schema_variants
    ON si_prop_belongs_to_root_prop.belongs_to_id = schema_variants.root_prop_id
       AND schema_variants.id = $4
) si_child_prop
  ON attribute_values.attribute_context_prop_id = si_child_prop.id
-- We will also take the "default" type too (corresponds to the attribute
-- value whose context has component id unset)
WHERE in_attribute_context_v1(
  attribute_context_build_from_parts_v1(
    si_child_prop.id, -- PropId
    ident_nil_v1(), -- InternalProviderId
    ident_nil_v1(), -- ExternalProviderId
    $3 -- ComponentId
  ),
  attribute_values
)
ORDER BY attribute_values.attribute_context_prop_id,
         attribute_values.attribute_context_component_id DESC
