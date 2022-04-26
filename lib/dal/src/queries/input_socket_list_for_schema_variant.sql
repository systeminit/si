SELECT DISTINCT ON (input_sockets.id) input_sockets.id,
                                      input_sockets.visibility_change_set_pk,
                                      input_sockets.visibility_edit_session_pk,
                                      row_to_json(input_sockets.*) AS object
FROM input_sockets
         INNER JOIN schema_variants ON
            schema_variants.id = input_sockets.schema_variant_id
        AND in_tenancy_v1($1, schema_variants.tenancy_universal,
                          schema_variants.tenancy_billing_account_ids,
                          schema_variants.tenancy_organization_ids,
                          schema_variants.tenancy_workspace_ids)
        AND is_visible_v1($2, schema_variants.visibility_change_set_pk,
                          schema_variants.visibility_edit_session_pk,
                          schema_variants.visibility_deleted)
WHERE in_tenancy_v1($1, input_sockets.tenancy_universal, input_sockets.tenancy_billing_account_ids,
                    input_sockets.tenancy_organization_ids, input_sockets.tenancy_workspace_ids)
  AND is_visible_v1($2, input_sockets.visibility_change_set_pk, input_sockets.visibility_edit_session_pk,
                    input_sockets.visibility_deleted)
  AND schema_variants.id = $3
ORDER BY input_sockets.id,
         visibility_change_set_pk DESC,
         visibility_edit_session_pk DESC
