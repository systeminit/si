SELECT DISTINCT ON (av.attribute_context_prop_id) row_to_json(av.*) AS object

FROM attribute_values_v1($1, $2) AS av
         JOIN (
    SELECT root_child_prop.id
    FROM props_v1($1, $2) AS root_child_prop
             JOIN prop_belongs_to_prop_v1($1, $2) AS pbtp
                  ON root_child_prop.name = $3
                      AND pbtp.object_id = root_child_prop.id
                      AND pbtp.belongs_to_id IN (
                          SELECT pmtmsv.left_object_id AS root_prop_id
                          FROM prop_many_to_many_schema_variants_v1($1, $2) AS pmtmsv
                                   JOIN component_belongs_to_schema_variant_v1($1, $2) AS cbtsv
                                        ON cbtsv.belongs_to_id = pmtmsv.right_object_id
                                            AND cbtsv.object_id = $4
                      )
) AS root_child_prop
              ON av.attribute_context_prop_id = root_child_prop.id

WHERE in_attribute_context_v1(
              attribute_context_build_from_parts_v1(
                      root_child_prop.id, -- PropId
                      -1, -- InternalProviderId
                      -1, -- ExternalProviderId
                      NULL, -- SchemaId (handled by ComponentId)
                      NULL, -- SchemaVariantId (handled by ComponentId)
                      $4 -- ComponentId
                  ),
              av
          )
ORDER BY av.attribute_context_prop_id,
         av.attribute_context_schema_id DESC,
         av.attribute_context_schema_variant_id DESC,
         av.attribute_context_component_id DESC,
         av.tenancy_universal
-- bools sort false first ascending.
