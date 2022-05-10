SELECT DISTINCT ON (code_generation_resolvers.id) code_generation_resolvers.id,
                                             code_generation_resolvers.visibility_change_set_pk,
                                             code_generation_resolvers.visibility_edit_session_pk,
                                             code_generation_resolvers.component_id,
                                             code_generation_resolvers.schema_id,
                                             code_generation_resolvers.schema_variant_id,
                                             code_generation_resolvers.system_id,
                                             row_to_json(code_generation_resolvers.*) as object
FROM code_generation_resolvers
WHERE in_tenancy_v1($1, code_generation_resolvers.tenancy_universal, code_generation_resolvers.tenancy_billing_account_ids, code_generation_resolvers.tenancy_organization_ids,
                    code_generation_resolvers.tenancy_workspace_ids)
  AND is_visible_v1($2, code_generation_resolvers.visibility_change_set_pk, code_generation_resolvers.visibility_edit_session_pk, code_generation_resolvers.visibility_deleted_at)
  AND code_generation_resolvers.code_generation_prototype_id = $3
  AND code_generation_resolvers.component_id = $4
ORDER BY code_generation_resolvers.id,
         visibility_change_set_pk DESC,
         visibility_edit_session_pk DESC,
         component_id DESC,
         system_id DESC,
         schema_variant_id DESC,
         schema_id DESC;
