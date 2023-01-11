select 
    validation_prototypes.id,
    row_to_json(validation_prototypes.*) as object 
from validation_prototypes_v1($1, $2) as validation_prototypes 
    where validation_prototypes.prop_id = $3
    and validation_prototypes.schema_variant_id = $4
    and validation_prototypes.schema_id = $5
