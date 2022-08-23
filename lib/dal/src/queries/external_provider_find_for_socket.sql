SELECT DISTINCT ON (external_providers.id) external_providers.id,
                                           external_providers.visibility_change_set_pk,
                                           external_providers.visibility_deleted_at,
                                           row_to_json(external_providers.*) AS object

FROM external_providers
         INNER JOIN socket_belongs_to_external_provider
                    ON external_providers.id = socket_belongs_to_external_provider.belongs_to_id
                        AND in_tenancy_v1($1,
                                          socket_belongs_to_external_provider.tenancy_universal,
                                          socket_belongs_to_external_provider.tenancy_billing_account_ids,
                                          socket_belongs_to_external_provider.tenancy_organization_ids,
                                          socket_belongs_to_external_provider.tenancy_workspace_ids)
                        AND is_visible_v1($2,
                                          socket_belongs_to_external_provider.visibility_change_set_pk,
                                          socket_belongs_to_external_provider.visibility_deleted_at)
                        AND socket_belongs_to_external_provider.object_id = $3
WHERE in_tenancy_v1($1,
                    external_providers.tenancy_universal,
                    external_providers.tenancy_billing_account_ids,
                    external_providers.tenancy_organization_ids,
                    external_providers.tenancy_workspace_ids)
  AND is_visible_v1($2,
                    external_providers.visibility_change_set_pk,
                    external_providers.visibility_deleted_at)

ORDER BY external_providers.id,
         external_providers.visibility_change_set_pk DESC,
         external_providers.visibility_deleted_at DESC NULLS FIRST;
