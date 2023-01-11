SELECT row_to_json(external_providers.*) AS object
FROM external_providers_v1($1, $2) AS external_providers
INNER JOIN attribute_prototypes_v1($1, $2) AS attribute_prototypes
    ON attribute_prototypes.id = external_prototypes.attribute_prototype_id
INNER JOIN attribute_prototype_arguments_v1($1, $2) AS attribute_prototype_arguments
    ON attribute_prototype_arguments.attribute_prototype_id = attribute_prototypes.id
WHERE attribute_prototype_arguments.internal_provider_id = $3
