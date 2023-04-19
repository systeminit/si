SELECT DISTINCT ON (av.attribute_context_prop_id) row_to_json(av.*) AS object

FROM attribute_values_v1($1, $2) AS av
         JOIN (
    SELECT root_child_prop.id
    FROM props_v1($1, $2) AS root_child_prop
             JOIN prop_belongs_to_prop_v1($1, $2) AS pbtp
                  ON root_child_prop.name = $3
                      AND pbtp.object_id = root_child_prop.id
             JOIN schema_variants_v1($1, $2) AS schema_variants
                  ON schema_variants.root_prop_id = pbtp.belongs_to_id
             JOIN component_belongs_to_schema_variant_v1($1, $2) AS cbtsv
                  ON cbtsv.belongs_to_id = schema_variants.id
                      AND cbtsv.object_id = $4
) AS root_child_prop
              ON av.attribute_context_prop_id = root_child_prop.id

WHERE in_attribute_context_v1(
              attribute_context_build_from_parts_v1(
                      root_child_prop.id, -- PropId
                      ident_nil_v1(), -- InternalProviderId
                      ident_nil_v1(), -- ExternalProviderId
                      $4 -- ComponentId
                  ),
              av
          )
ORDER BY av.attribute_context_prop_id,
         av.attribute_context_component_id DESC
