SELECT DISTINCT ON (internal_providers.id) internal_providers.id,
                                           internal_providers.visibility_change_set_pk,
                                           internal_providers.visibility_edit_session_pk,
                                           row_to_json(internal_providers.*) AS object
FROM internal_providers
        INNER JOIN schema_variants ON
            schema_variants.id = internal_providers.schema_variant_id
        AND in_tenancy_v1($1, schema_variants.tenancy_universal,
                          schema_variants.tenancy_billing_account_ids,
                          schema_variants.tenancy_organization_ids,
                          schema_variants.tenancy_workspace_ids)
        AND is_visible_v1($2, schema_variants.visibility_change_set_pk,
                          schema_variants.visibility_edit_session_pk,
                          schema_variants.visibility_deleted_at)
WHERE in_tenancy_v1($1, internal_providers.tenancy_universal, internal_providers.tenancy_billing_account_ids,
                    internal_providers.tenancy_organization_ids, internal_providers.tenancy_workspace_ids)
  AND is_visible_v1($2, internal_providers.visibility_change_set_pk, internal_providers.visibility_edit_session_pk,
                    internal_providers.visibility_deleted_at)
  AND schema_variants.id = $3
ORDER BY internal_providers.id,
         visibility_change_set_pk DESC,
         visibility_edit_session_pk DESC
