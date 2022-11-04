SELECT DISTINCT ON (av.attribute_context_prop_id) row_to_json(av.*) AS object

FROM attribute_values_v1($1, $2) AS av
         JOIN (
    SELECT resource_prop.id
    FROM props_v1($1, $2) AS resource_prop
             JOIN prop_belongs_to_prop_v1($1, $2) AS pbtp
                  ON resource_prop.name = 'resource'
                      AND pbtp.object_id = resource_prop.id
                      AND pbtp.belongs_to_id IN (
                          SELECT pmtmsv.left_object_id AS root_prop_id
                          FROM prop_many_to_many_schema_variants_v1($1, $2) AS pmtmsv
                                   JOIN component_belongs_to_schema_variant_v1($1, $2) AS cbtsv
                                        ON cbtsv.belongs_to_id = pmtmsv.right_object_id
                                            AND cbtsv.object_id = $3
                      )
) AS resource_prop
              ON av.attribute_context_prop_id = resource_prop.id

WHERE in_attribute_context_v1(
              attribute_context_build_from_parts_v1(
                      resource_prop.id, -- PropId
                      -1, -- InternalProviderId
                      -1, -- ExternalProviderId
                      NULL, -- SchemaId (handled by ComponentId)
                      NULL, -- SchemaVariantId (handled by ComponentId)
                      $3, -- ComponentId
                      -1 -- SystemId (NOTE(nick,jacob): system is going away, we never set it)
                  ),
              av
          )
ORDER BY av.attribute_context_prop_id,
         av.attribute_context_schema_id DESC,
         av.attribute_context_schema_variant_id DESC,
         av.attribute_context_component_id DESC,
         av.attribute_context_system_id DESC,
         av.tenancy_universal
-- bools sort false first ascending.