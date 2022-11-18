SELECT row_to_json(external_providers.*) AS object
FROM external_providers_v1($1, $2) AS external_providers
INNER JOIN socket_belongs_to_external_provider_v1($1, $2) AS socket_belongs_to_external_provider
    ON external_providers.id = socket_belongs_to_external_provider.belongs_to_id
        AND socket_belongs_to_external_provider.object_id = $3
