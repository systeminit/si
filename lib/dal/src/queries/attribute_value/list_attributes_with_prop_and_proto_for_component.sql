SELECT av.id as attribute_value_id, p.id as prop_id, avbtap.belongs_to_id as prototype_id
  FROM attribute_values_v1($1, $2) AS av
    inner join attribute_value_belongs_to_attribute_prototype_v1($1, $2) as avbtap
      on av.id = avbtap.object_id
    inner join props_v1($1, $2) as p
      on p.id = av.attribute_context_prop_id
    where in_attribute_context_v1($3, av) and av.attribute_context_component_id = $4
