SELECT DISTINCT ON (output_sockets.id) output_sockets.id,
                                      output_sockets.visibility_change_set_pk,
                                      output_sockets.visibility_edit_session_pk,
                                      row_to_json(output_sockets.*) AS object
FROM output_sockets
         INNER JOIN schema_variants ON
            schema_variants.id = output_sockets.schema_variant_id
        AND in_tenancy_v1($1, schema_variants.tenancy_universal,
                          schema_variants.tenancy_billing_account_ids,
                          schema_variants.tenancy_organization_ids,
                          schema_variants.tenancy_workspace_ids)
        AND is_visible_v1($2, schema_variants.visibility_change_set_pk,
                          schema_variants.visibility_edit_session_pk,
                          schema_variants.visibility_deleted)
WHERE in_tenancy_v1($1, output_sockets.tenancy_universal, output_sockets.tenancy_billing_account_ids,
                    output_sockets.tenancy_organization_ids, output_sockets.tenancy_workspace_ids)
  AND is_visible_v1($2, output_sockets.visibility_change_set_pk, output_sockets.visibility_edit_session_pk,
                    output_sockets.visibility_deleted)
  AND schema_variants.id = $3
ORDER BY output_sockets.id,
         visibility_change_set_pk DESC,
         visibility_edit_session_pk DESC
