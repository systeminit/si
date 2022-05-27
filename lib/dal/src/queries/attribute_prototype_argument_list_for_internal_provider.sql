SELECT DISTINCT ON (attribute_prototype_arguments.id) attribute_prototype_arguments.id,
                                                      attribute_prototype_arguments.visibility_change_set_pk,
                                                      attribute_prototype_arguments.visibility_edit_session_pk,
                                                      attribute_prototype_arguments.visibility_deleted_at,
                                                      attribute_prototype_arguments.name,
                                                      attribute_prototype_arguments.internal_provider_id,
                                                      row_to_json(attribute_prototype_arguments.*) AS object
FROM attribute_prototype_arguments
WHERE in_tenancy_v1($1, attribute_prototype_arguments.tenancy_universal,
                    attribute_prototype_arguments.tenancy_billing_account_ids,
                    attribute_prototype_arguments.tenancy_organization_ids,
                    attribute_prototype_arguments.tenancy_workspace_ids)
  AND is_visible_v1($2, attribute_prototype_arguments.visibility_change_set_pk,
                    attribute_prototype_arguments.visibility_edit_session_pk,
                    attribute_prototype_arguments.visibility_deleted_at)
  AND attribute_prototype_arguments.internal_provider_id = $3

ORDER BY attribute_prototype_arguments.id,
         visibility_change_set_pk DESC,
         visibility_edit_session_pk DESC,
         visibility_deleted_at DESC NULLS FIRST;
