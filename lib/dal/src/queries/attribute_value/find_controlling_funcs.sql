WITH recursive parent_funcs as (
  SELECT 1 as depth, funcs.name, func_id, avbtap.object_id as attribute_value_id
    from attribute_prototypes_v1($1, $2) as ap
      inner join attribute_value_belongs_to_attribute_prototype_v1($1, $2) as avbtap
        on ap.id = avbtap.belongs_to_id
      inner join funcs_v1($1, $2) as funcs
        on funcs.id = ap.func_id
    where avbtap.object_id = $3
  union all
  SELECT depth + 1 as depth, funcs.name, ap.func_id, avbtap.object_id as attribute_value_id
    from attribute_prototypes_v1($1, $2) as ap
      inner join attribute_value_belongs_to_attribute_prototype_v1($1, $2) as avbtap
        on ap.id = avbtap.belongs_to_id
      inner join attribute_value_belongs_to_attribute_value_v1($1, $2) as avbtav
        on avbtav.belongs_to_id = avbtap.object_id
      inner join parent_funcs on parent_funcs.attribute_value_id = avbtav.object_id
      inner join funcs_v1($1, $2) as funcs
        on funcs.id = ap.func_id
)
SELECT row_to_json(parent_funcs.*) as object FROM parent_funcs