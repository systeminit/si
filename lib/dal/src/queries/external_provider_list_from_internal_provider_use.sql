SELECT DISTINCT ON (external_providers.id) external_providers.id,
                                           external_providers.visibility_change_set_pk,
                                           external_providers.visibility_edit_session_pk,
                                           external_providers.visibility_deleted_at,
                                           row_to_json(external_providers.*) AS object
FROM external_proivders
         INNER JOIN attribute_prototypes
                    ON attribute_prototypes.id = external_prototypes.attribute_prototype_id
                        AND is_visible_v1($2, attribute_prototypes.visibility_change_set_pk,
                                          attribute_prototypes.visibility_edit_session_pk,
                                          attribute_prototypes.visibility_deleted_at)
         INNER JOIN attribute_prototype_arguments
                    ON attribute_prototype_arguments.attribute_prototype_id = attribute_prototypes.id
                        AND is_visible_v1($2, attribute_prototype_arguments.visibility_change_set_pk,
                                          attribute_prototype_arguments.visibility_edit_session_pk,
                                          attribute_prototype_arguments.visibility_deleted_at)

WHERE in_tenancy_v1($1, external_providers.tenancy_universal, external_providers.tenancy_billing_account_ids,
                    external_providers.tenancy_organization_ids, external_providers.tenancy_workspace_ids)
  AND is_visible_v1($2, external_providers.visibility_change_set_pk, external_providers.visibility_edit_session_pk,
                    external_providers.visibility_deleted_at)
  AND attribute_prototype_arguments.internal_provider_id = $3

ORDER BY external_providers.id,
         external_providers.visibility_change_set_pk DESC,
         external_providers.visibility_edit_session_pk DESC,
         external_providers.visibility_deleted_at DESC NULLS FIRST;
