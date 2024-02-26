SELECT p.id as prop_id, avbtap.belongs_to_id as prototype_id
  FROM attribute_values_v1($1, $2) AS av
    inner join attribute_value_belongs_to_attribute_prototype_v1($1, $2) as avbtap
      on av.id = avbtap.object_id
    inner join component_belongs_to_schema_variant_v1($1, $2) as cbtsv
      on cbtsv.object_id = $4
    inner join props_v1($1, $2) as p
      on p.id = av.attribute_context_prop_id
    where in_attribute_context_v1($3, av) and p.schema_variant_id = cbtsv.belongs_to_id
