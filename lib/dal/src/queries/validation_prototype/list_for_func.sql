select validation_prototypes.id,
    row_to_json(validation_prototypes.*) as object
from validation_prototypes_v1($1, $2) as validation_prototypes
where validation_prototypes.func_id = $3;
