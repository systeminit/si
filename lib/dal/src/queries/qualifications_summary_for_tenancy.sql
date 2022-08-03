SELECT component_id,
       component_name,
       count(qualification_id)                                         as total_qualifications,
       sum(case when qualification_status = 'true' then 1 else 0 end)  as succeeded,
       sum(case when qualification_status = 'false' then 1 else 0 end) as failed
FROM (SELECT DISTINCT ON (components.component_id, qualification_resolvers.id) components.component_id,
                                                                               components.prop_values -> 'si' ->> 'name'      as component_name,
                                                                               qualification_resolvers.id                     as qualification_id,
                                                                               func_binding_return_values.value ->> 'success' as qualification_status
      FROM components_with_attributes components
               LEFT JOIN qualification_resolvers ON components.component_id = qualification_resolvers.component_id
               LEFT JOIN func_binding_return_value_belongs_to_func_binding ON
              func_binding_return_value_belongs_to_func_binding.belongs_to_id =
              qualification_resolvers.func_binding_id
               LEFT JOIN func_binding_return_values ON
              func_binding_return_values.id = func_binding_return_value_belongs_to_func_binding.object_id
               LEFT JOIN qualification_prototypes ON
              qualification_prototypes.id = qualification_resolvers.qualification_prototype_id
               LEFT JOIN component_belongs_to_schema ON
              components.component_id = component_belongs_to_schema.object_id
               LEFT JOIN schemas ON
              component_belongs_to_schema.belongs_to_id = schemas.id
      WHERE in_tenancy_v1($1, components.tenancy_universal,
                          components.tenancy_billing_account_ids,
                          components.tenancy_organization_ids,
                          components.tenancy_workspace_ids)
        AND is_visible_v1($2, components.visibility_change_set_pk,
                          components.visibility_edit_session_pk, components.visibility_deleted_at)
        AND schemas.kind != 'concept'
      ORDER BY components.component_id, qualification_resolvers.id,
               qualification_prototypes.visibility_change_set_pk DESC,
               qualification_prototypes.visibility_edit_session_pk DESC,
               qualification_resolvers.visibility_change_set_pk DESC,
               qualification_resolvers.visibility_edit_session_pk DESC) as qualification_data
GROUP BY component_id, component_name

