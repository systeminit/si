SELECT DISTINCT ON (attribute_values.id) attribute_values.id,
                                         attribute_values.visibility_change_set_pk,
                                         attribute_values.visibility_edit_session_pk,
                                         attribute_values.visibility_deleted_at,
                                         row_to_json(attribute_values.*) AS object

FROM attribute_values
         INNER JOIN attribute_value_belongs_to_attribute_prototype
                    ON attribute_value_belongs_to_attribute_prototype.object_id = attribute_values.id
                        AND is_visible_v1($2, attribute_value_belongs_to_attribute_prototype.visibility_change_set_pk,
                                          attribute_value_belongs_to_attribute_prototype.visibility_edit_session_pk,
                                          attribute_value_belongs_to_attribute_prototype.visibility_deleted_at)

         INNER JOIN attribute_prototype_argument_belongs_to_attribute_prototype
                    ON attribute_prototype_argument_belongs_to_attribute_prototype.belongs_to_id =
                       attribute_value_belongs_to_attribute_prototype.belongs_to_id
                        AND is_visible_v1($2,
                                          attribute_prototype_argument_belongs_to_attribute_prototype.visibility_change_set_pk,
                                          attribute_prototype_argument_belongs_to_attribute_prototype.visibility_edit_session_pk,
                                          attribute_prototype_argument_belongs_to_attribute_prototype.visibility_deleted_at)

         INNER JOIN attribute_prototype_arguments
                    ON attribute_prototype_arguments.id =
                       attribute_prototype_argument_belongs_to_attribute_prototype.object_id
                        AND is_visible_v1($2, attribute_prototype_arguments.visibility_change_set_pk,
                                          attribute_prototype_arguments.visibility_edit_session_pk,
                                          attribute_prototype_arguments.visibility_deleted_at)

WHERE in_tenancy_v1($1, attribute_values.tenancy_universal, attribute_values.tenancy_billing_account_ids,
                    attribute_values.tenancy_organization_ids, attribute_values.tenancy_workspace_ids)
  AND is_visible_v1($2, attribute_values.visibility_change_set_pk, attribute_values.visibility_edit_session_pk,
                    attribute_values.visibility_deleted_at)
  AND attribute_prototype_arguments.internal_provider_id = $3

ORDER BY attribute_values.id,
         attribute_values.visibility_change_set_pk DESC,
         attribute_values.visibility_edit_session_pk DESC,
         attribute_values.visibility_deleted_at DESC NULLS FIRST;