SELECT DISTINCT ON
    (internal_providers.id) internal_providers.id,
                            internal_providers.visibility_change_set_pk,
                            internal_providers.visibility_deleted_at,
                            row_to_json(internal_providers.*) AS object
FROM internal_providers
         JOIN socket_belongs_to_internal_provider sbtip on internal_providers.id = sbtip.belongs_to_id
    AND in_tenancy_and_visible_v1($1, $2, sbtip)
         JOIN sockets ON sockets.id = sbtip.object_id
    AND in_tenancy_and_visible_v1($1, $2, sockets)
WHERE in_tenancy_and_visible_v1($1, $2, internal_providers)
  AND internal_providers.prop_id = -1
ORDER BY internal_providers.id,
         internal_providers.visibility_change_set_pk DESC,
         internal_providers.visibility_deleted_at DESC NULLS FIRST;