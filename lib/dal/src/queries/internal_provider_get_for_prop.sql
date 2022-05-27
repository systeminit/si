SELECT DISTINCT ON (internal_providers.id) internal_providers.id,
                                           internal_providers.visibility_change_set_pk,
                                           internal_providers.visibility_edit_session_pk,
                                           internal_providers.visibility_deleted_at,
                                           row_to_json(internal_providers.*) AS object
FROM internal_providers
WHERE in_tenancy_v1($1, internal_providers.tenancy_universal, internal_providers.tenancy_billing_account_ids,
                    internal_providers.tenancy_organization_ids, internal_providers.tenancy_workspace_ids)
  AND is_visible_v1($2, internal_providers.visibility_change_set_pk, internal_providers.visibility_edit_session_pk,
                    internal_providers.visibility_deleted_at)
  AND prop_id = $3
ORDER BY internal_providers.id,
         visibility_change_set_pk DESC,
         visibility_edit_session_pk DESC,
         visibility_deleted_at DESC NULLS FIRST;
