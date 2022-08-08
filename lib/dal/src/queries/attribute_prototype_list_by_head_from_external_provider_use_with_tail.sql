SELECT DISTINCT ON (attribute_prototypes.id, attribute_prototype_arguments.head_component_id) attribute_prototypes.id,
                                                                                              attribute_prototype_arguments.head_component_id AS head_component_id,
                                                                                              attribute_prototypes.visibility_change_set_pk,
                                                                                              attribute_prototypes.visibility_deleted_at,
                                                                                              attribute_prototype_arguments.visibility_change_set_pk,
                                                                                              attribute_prototype_arguments.visibility_deleted_at,

                                                                                              row_to_json(attribute_prototypes.*)             AS object

FROM attribute_prototypes
         INNER JOIN attribute_prototype_arguments
                    ON attribute_prototype_arguments.attribute_prototype_id = attribute_prototypes.id
                        AND is_visible_v1($2, attribute_prototype_arguments.visibility_change_set_pk,
                                          attribute_prototype_arguments.visibility_deleted_at)

WHERE in_tenancy_v1($1, attribute_prototypes.tenancy_universal, attribute_prototypes.tenancy_billing_account_ids,
                    attribute_prototypes.tenancy_organization_ids, attribute_prototypes.tenancy_workspace_ids)
  AND is_visible_v1($2, attribute_prototypes.visibility_change_set_pk,
                    attribute_prototypes.visibility_deleted_at)
  AND attribute_prototype_arguments.external_provider_id = $3
  AND attribute_prototype_arguments.tail_component_id = $4

ORDER BY attribute_prototypes.id,
         attribute_prototype_arguments.head_component_id DESC,
         attribute_prototypes.visibility_change_set_pk DESC,
         attribute_prototypes.visibility_deleted_at DESC NULLS FIRST,
         attribute_prototype_arguments.visibility_change_set_pk DESC,
         attribute_prototype_arguments.visibility_deleted_at DESC NULLS FIRST;
