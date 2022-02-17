SELECT count(*) > 0 AS siblings_are_set
FROM (
         SELECT DISTINCT ON (sibling_attribute_resolvers.id) sibling_attribute_resolvers.id,
                                                             sibling_attribute_resolvers.prop_id,
                                                             sibling_attribute_resolvers.visibility_change_set_pk,
                                                             sibling_attribute_resolvers.visibility_edit_session_pk,
                                                             sibling_attribute_resolvers.component_id,
                                                             sibling_attribute_resolvers.schema_id,
                                                             sibling_attribute_resolvers.schema_variant_id,
                                                             sibling_attribute_resolvers.system_id,
                                                             func_bindings.backend_kind
         FROM
             -- Our original attribute resolver
             attribute_resolvers AS ar
                 -- Which attribute resolver is it a child of
                 INNER JOIN attribute_resolver_belongs_to_attribute_resolver AS arbtar ON
                         ar.id = arbtar.object_id
                     AND in_tenancy_v1($1, arbtar.tenancy_universal, arbtar.tenancy_billing_account_ids,
                                       arbtar.tenancy_organization_ids, arbtar.tenancy_workspace_ids)
                     AND is_visible_v1($2, arbtar.visibility_change_set_pk, arbtar.visibility_edit_session_pk,
                                       arbtar.visibility_deleted)
                 -- Which attribute resolvers are also children of that attribute resolver
                 INNER JOIN attribute_resolver_belongs_to_attribute_resolver AS sibling_belongs_to ON
                         arbtar.belongs_to_id = sibling_belongs_to.belongs_to_id
                     AND sibling_belongs_to.object_id != $3
                     AND in_tenancy_v1($1, sibling_belongs_to.tenancy_universal,
                                       sibling_belongs_to.tenancy_billing_account_ids,
                                       sibling_belongs_to.tenancy_organization_ids,
                                       sibling_belongs_to.tenancy_workspace_ids)
                     AND is_visible_v1($2, sibling_belongs_to.visibility_change_set_pk,
                                       sibling_belongs_to.visibility_edit_session_pk,
                                       sibling_belongs_to.visibility_deleted)
                 -- The sibling child resolvers
                 INNER JOIN attribute_resolvers AS sibling_attribute_resolvers ON
                         sibling_attribute_resolvers.id = sibling_belongs_to.object_id
                     AND in_tenancy_v1($1, sibling_attribute_resolvers.tenancy_universal,
                                       sibling_attribute_resolvers.tenancy_billing_account_ids,
                                       sibling_attribute_resolvers.tenancy_organization_ids,
                                       sibling_attribute_resolvers.tenancy_workspace_ids)
                     AND is_visible_v1($2, sibling_attribute_resolvers.visibility_change_set_pk,
                                       sibling_attribute_resolvers.visibility_edit_session_pk,
                                       sibling_attribute_resolvers.visibility_deleted)
                 -- The sibling's func bindings
                 INNER JOIN func_bindings ON
                         func_bindings.id = sibling_attribute_resolvers.func_binding_id
                     AND in_tenancy_v1($1, func_bindings.tenancy_universal, func_bindings.tenancy_billing_account_ids,
                                       func_bindings.tenancy_organization_ids, func_bindings.tenancy_workspace_ids)
                     AND is_visible_v1($2, func_bindings.visibility_change_set_pk,
                                       func_bindings.visibility_edit_session_pk,
                                       func_bindings.visibility_deleted)
         WHERE in_tenancy_v1($1, ar.tenancy_universal, ar.tenancy_billing_account_ids, ar.tenancy_organization_ids,
                             ar.tenancy_workspace_ids)
           AND is_visible_v1($2, ar.visibility_change_set_pk, ar.visibility_edit_session_pk, ar.visibility_deleted)
           AND ar.id = $3
         ORDER BY sibling_attribute_resolvers.id,
                  prop_id,
                  visibility_change_set_pk DESC,
                  visibility_edit_session_pk DESC,
                  component_id DESC,
                  system_id DESC,
                  schema_variant_id DESC,
                  schema_id DESC
     ) AS sibling_values
WHERE sibling_values.backend_kind != 'Unset';
