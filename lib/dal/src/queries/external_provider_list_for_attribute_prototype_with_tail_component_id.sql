SELECT DISTINCT ON (external_providers.id) external_providers.id,
                                           external_providers.visibility_change_set_pk,
                                           external_providers.visibility_edit_session_pk,
                                           external_providers.visibility_deleted_at,
                                           row_to_json(external_providers.*) AS object
FROM external_providers
         INNER JOIN attribute_prototype_arguments ON
            attribute_prototype_arguments.external_provider_id = external_providers.id
        AND is_visible_v1($2, attribute_prototype_arguments.visibility_change_set_pk,
                          attribute_prototype_arguments.visibility_edit_session_pk,
                          attribute_prototype_arguments.visibility_deleted_at)

WHERE in_tenancy_v1($1, external_providers.tenancy_universal, external_providers.tenancy_billing_account_ids,
                    external_providers.tenancy_organization_ids, external_providers.tenancy_workspace_ids)
  AND is_visible_v1($2, external_providers.visibility_change_set_pk, external_providers.visibility_edit_session_pk,
                    external_providers.visibility_deleted_at)
  AND external_providers.attribute_prototype_id = $3
  AND attribute_prototype_arguments.tail_component_id = $4

ORDER BY external_providers.id,
         external_providers.visibility_change_set_pk DESC,
         external_providers.visibility_edit_session_pk DESC,
         external_providers.visibility_deleted_at DESC NULLS FIRST;
