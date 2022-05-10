SELECT DISTINCT ON (attribute_prototype_arguments.id) attribute_prototype_arguments.id,
                                           attribute_prototype_arguments.visibility_change_set_pk,
                                           attribute_prototype_arguments.visibility_edit_session_pk,
                                           attribute_prototype_arguments.name,
                                           attribute_prototype_arguments.internal_provider_id,
                                           row_to_json(attribute_prototype_arguments.*) AS object
FROM attribute_prototype_arguments

    INNER JOIN attribute_prototype_argument_belongs_to_attribute_prototype ON
               attribute_prototype_argument_belongs_to_attribute_prototype.object_id = attribute_prototype_arguments.id
        AND in_tenancy_v1($1, attribute_prototype_argument_belongs_to_attribute_prototype.tenancy_universal,
                          attribute_prototype_argument_belongs_to_attribute_prototype.tenancy_billing_account_ids,
                          attribute_prototype_argument_belongs_to_attribute_prototype.tenancy_organization_ids,
                          attribute_prototype_argument_belongs_to_attribute_prototype.tenancy_workspace_ids)
        AND is_visible_v1($2, attribute_prototype_argument_belongs_to_attribute_prototype.visibility_change_set_pk,
                          attribute_prototype_argument_belongs_to_attribute_prototype.visibility_edit_session_pk,
                          attribute_prototype_argument_belongs_to_attribute_prototype.visibility_deleted_at)
    INNER JOIN attribute_prototypes ON
               attribute_prototypes.id = attribute_prototype_argument_belongs_to_attribute_prototype.belongs_to_id
        AND in_tenancy_v1($1, attribute_prototypes.tenancy_universal,
                          attribute_prototypes.tenancy_billing_account_ids,
                          attribute_prototypes.tenancy_organization_ids,
                          attribute_prototypes.tenancy_workspace_ids)
        AND is_visible_v1($2, attribute_prototypes.visibility_change_set_pk,
                          attribute_prototypes.visibility_edit_session_pk,
                          attribute_prototypes.visibility_deleted_at)
        AND attribute_prototypes.id = $3

WHERE in_tenancy_v1($1, attribute_prototype_arguments.tenancy_universal, attribute_prototype_arguments.tenancy_billing_account_ids,
                    attribute_prototype_arguments.tenancy_organization_ids, attribute_prototype_arguments.tenancy_workspace_ids)
  AND is_visible_v1($2, attribute_prototype_arguments.visibility_change_set_pk, attribute_prototype_arguments.visibility_edit_session_pk,
                    attribute_prototype_arguments.visibility_deleted_at)

ORDER BY attribute_prototype_arguments.id,
         visibility_change_set_pk DESC,
         visibility_edit_session_pk DESC
