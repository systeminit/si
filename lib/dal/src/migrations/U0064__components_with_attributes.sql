-- TODO: Convert this to a SQL function that takes tenancy & visibility so it can use the table functions
CREATE OR REPLACE VIEW components_with_attributes AS
SELECT components.id                                     AS component_id,
       component_belongs_to_schema.belongs_to_id         AS schema_id,
       schemas.name                                      AS schema_name,
       component_belongs_to_schema_variant.belongs_to_id AS schema_variant_id,
       schema_variants.name                              AS schema_variant_name,
       prop_many_to_many_schema_variants.left_object_id  AS root_prop_id,
       internal_providers.id                             AS internal_provider_id,
       attribute_values.id                               AS attribute_value_id,
       func_binding_return_values.tenancy_universal,
       func_binding_return_values.tenancy_billing_account_ids,
       func_binding_return_values.tenancy_organization_ids,
       func_binding_return_values.tenancy_workspace_ids,
       func_binding_return_values.visibility_change_set_pk,
       func_binding_return_values.visibility_deleted_at,
       func_binding_return_values.id                     AS func_binding_return_value_id,
       func_binding_return_values.value                  AS prop_values
FROM components
         LEFT JOIN component_belongs_to_schema ON component_belongs_to_schema.object_id = components.id
         LEFT JOIN component_belongs_to_schema_variant ON component_belongs_to_schema_variant.object_id = components.id
         LEFT JOIN schemas ON schemas.id = component_belongs_to_schema.belongs_to_id
         LEFT JOIN schema_variants ON schema_variants.id = component_belongs_to_schema_variant.belongs_to_id
         LEFT JOIN prop_many_to_many_schema_variants
                   ON prop_many_to_many_schema_variants.right_object_id = schema_variants.id
         LEFT JOIN internal_providers ON internal_providers.prop_id = prop_many_to_many_schema_variants.left_object_id
         LEFT JOIN attribute_values
                   ON attribute_values.attribute_context_internal_provider_id = internal_providers.id AND
                      attribute_values.attribute_context_component_id = components.id
         LEFT JOIN func_binding_return_values
                   ON func_binding_return_values.id = attribute_values.func_binding_return_value_id;
