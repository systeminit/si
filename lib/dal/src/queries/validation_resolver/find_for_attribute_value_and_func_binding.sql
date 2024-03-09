select row_to_json(validation_resolvers.*) as object
from validation_resolvers_v1($1, $2) as validation_resolvers
where attribute_value_id = $3
  and validation_func_id = $4
order by validation_resolvers.id desc
