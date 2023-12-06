SELECT bt.object_id, row_to_json(external_providers.*)
FROM socket_belongs_to_external_provider_v1($1, $2) bt
INNER JOIN external_providers_v1($1, $2) external_providers ON external_providers.id = bt.belongs_to_id
