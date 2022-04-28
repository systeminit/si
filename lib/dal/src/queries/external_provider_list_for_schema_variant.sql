SELECT DISTINCT ON (external_providers.id) external_providers.id,
                                       external_providers.visibility_change_set_pk,
                                       external_providers.visibility_edit_session_pk,
                                       row_to_json(external_providers.*) AS object
FROM external_providers
        INNER JOIN schema_variants ON
            schema_variants.id = external_providers.schema_variant_id
        AND in_tenancy_v1($1, schema_variants.tenancy_universal,
                          schema_variants.tenancy_billing_account_ids,
                          schema_variants.tenancy_organization_ids,
                          schema_variants.tenancy_workspace_ids)
        AND is_visible_v1($2, schema_variants.visibility_change_set_pk,
                          schema_variants.visibility_edit_session_pk,
                          schema_variants.visibility_deleted)
WHERE in_tenancy_v1($1, external_providers.tenancy_universal, external_providers.tenancy_billing_account_ids,
                    external_providers.tenancy_organization_ids, external_providers.tenancy_workspace_ids)
  AND is_visible_v1($2, external_providers.visibility_change_set_pk, external_providers.visibility_edit_session_pk,
                    external_providers.visibility_deleted)
  AND schema_variants.id = $3
ORDER BY external_providers.id,
         visibility_change_set_pk DESC,
         visibility_edit_session_pk DESC