SELECT av.id as attribute_value_id, avbtap.belongs_to_id != prop_avbtap.belongs_to_id as overridden
  FROM attribute_values_v1($1, $2) AS av
    inner join attribute_value_belongs_to_attribute_prototype_v1($1, $2) as avbtap
      on av.id = avbtap.object_id
    inner join props_v1($1, $2) as p
      on p.id = av.attribute_context_prop_id
    inner join attribute_values_v1($1, $2) as prop_av
      on prop_av.attribute_context_prop_id = p.id and in_attribute_context_v1($3, prop_av)
    inner join attribute_value_belongs_to_attribute_prototype_v1($1, $2) as prop_avbtap
      on prop_avbtap.object_id = prop_av.id
    where in_attribute_context_v1($4, av) and av.attribute_context_component_id = $5