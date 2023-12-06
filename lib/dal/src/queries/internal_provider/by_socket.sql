SELECT bt.object_id, row_to_json(internal_providers.*)
FROM socket_belongs_to_internal_provider_v1($1, $2) bt
INNER JOIN internal_providers_v1($1, $2) internal_providers ON internal_providers.id = bt.belongs_to_id
