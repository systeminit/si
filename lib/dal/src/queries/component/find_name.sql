SELECT fbrv.value AS component_name
FROM func_binding_return_values_v1($1, $2) AS fbrv
WHERE id IN (
    SELECT DISTINCT ON (av.attribute_context_prop_id) av.func_binding_return_value_id
    FROM attribute_values_v1($1, $2) AS av
             JOIN (
        SELECT name_prop.id
        FROM props_v1($1, $2) AS name_prop
                 JOIN prop_belongs_to_prop_v1($1, $2) AS pbtp
                      ON name_prop.name = 'name'
                          AND pbtp.object_id = name_prop.id
                          AND pbtp.belongs_to_id IN (
                              SELECT si_prop.id
                              FROM props_v1($1, $2) AS si_prop
                                       JOIN prop_belongs_to_prop_v1($1, $2) AS pbtp
                                            ON si_prop.name = 'si'
                                                AND pbtp.object_id = si_prop.id
                                       JOIN schema_variants_v1($1, $2) as schema_variants
                                            ON pbtp.belongs_to_id = schema_variants.root_prop_id
                                       JOIN component_belongs_to_schema_variant_v1($1, $2) AS cbtsv
                                            ON cbtsv.belongs_to_id = schema_variants.id
                                                AND cbtsv.object_id = $3
                          )
    ) AS name_prop
                  ON av.attribute_context_prop_id = name_prop.id
    WHERE in_attribute_context_v1(
                  attribute_context_build_from_parts_v1(
                          name_prop.id, -- PropId
                          ident_nil_v1(), -- InternalProviderId
                          ident_nil_v1(), -- ExternalProviderId
                          $3 -- ComponentId
                      ),
                  av
              )
    ORDER BY av.attribute_context_prop_id,
             av.attribute_context_component_id DESC
)