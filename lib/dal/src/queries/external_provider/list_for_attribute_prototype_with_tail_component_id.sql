SELECT row_to_json(external_providers.*) AS object
FROM external_providers_v1($1, $2) AS external_providers
    INNER JOIN attribute_prototype_arguments_v1($1, $2)
        ON attribute_prototype_arguments.external_provider_id = external_providers.id
WHERE
    external_providers.attribute_prototype_id = $3
    AND attribute_prototype_arguments.tail_component_id = $4
