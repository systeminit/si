SELECT DISTINCT ON (component_id) component_id,
                                  components.prop_values -> 'si' ->> 'name' AS component_name

FROM components_with_attributes AS components

         -- Collect all unique component ids
         INNER JOIN (SELECT DISTINCT ON (attribute_context_component_id) attribute_context_component_id
                     FROM attribute_values

                          -- Grab all components on HEAD
                     WHERE attribute_context_component_id IN (SELECT id
                                                              FROM components
                                                              WHERE visibility_change_set_pk = ident_nil_v1()
                                                                AND visibility_deleted_at IS NULL
                                                                AND in_tenancy_v1($1,
                                                                                  tenancy_billing_account_pks,
                                                                                  tenancy_organization_pks,
                                                                                  tenancy_workspace_pks))

                       -- Compare only to the current change set
                       AND visibility_change_set_pk = $2

                       -- Ensure they are not deleted
                       AND visibility_deleted_at IS NULL

                       -- Scope the tenancy one last time
                       AND in_tenancy_v1($1,
                                         tenancy_billing_account_pks,
                                         tenancy_organization_pks,
                                         tenancy_workspace_pks)


                     ORDER BY attribute_context_component_id DESC) AS attribute_values
                    ON components.component_id = attribute_values.attribute_context_component_id

WHERE in_tenancy_v1($1,
                    tenancy_billing_account_pks,
                    tenancy_organization_pks,
                    tenancy_workspace_pks)

ORDER BY component_id DESC,
         component_name DESC
