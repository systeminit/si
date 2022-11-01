SELECT DISTINCT ON (internal_providers.id) internal_providers.id,
                                           internal_providers.visibility_change_set_pk,
                                           internal_providers.visibility_deleted_at,
                                           row_to_json(internal_providers.*) AS object

FROM internal_providers
         INNER JOIN socket_belongs_to_internal_provider
                    ON internal_providers.id = socket_belongs_to_internal_provider.belongs_to_id
                        AND in_tenancy_v1($1,
                                          socket_belongs_to_internal_provider.tenancy_universal,
                                          socket_belongs_to_internal_provider.tenancy_billing_account_ids,
                                          socket_belongs_to_internal_provider.tenancy_organization_ids,
                                          socket_belongs_to_internal_provider.tenancy_workspace_ids)
                        AND is_visible_v1($2,
                                          socket_belongs_to_internal_provider.visibility_change_set_pk,
                                          socket_belongs_to_internal_provider.visibility_deleted_at)
                        AND socket_belongs_to_internal_provider.object_id = $3
WHERE in_tenancy_v1($1,
                    internal_providers.tenancy_universal,
                    internal_providers.tenancy_billing_account_ids,
                    internal_providers.tenancy_organization_ids,
                    internal_providers.tenancy_workspace_ids)
  AND is_visible_v1($2,
                    internal_providers.visibility_change_set_pk,
                    internal_providers.visibility_deleted_at)

ORDER BY internal_providers.id,
         internal_providers.visibility_change_set_pk DESC,
         internal_providers.visibility_deleted_at DESC NULLS FIRST;
