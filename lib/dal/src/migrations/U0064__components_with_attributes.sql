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
       func_binding_return_values.tenancy_billing_account_pks,
       func_binding_return_values.tenancy_organization_pks,
       func_binding_return_values.tenancy_workspace_pks,
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

-- We need to create the following tenancy and visibility related functions by hand
-- because we're trying to pretend that the components_with_attributes view is a
-- "normal" standard model table.
CREATE OR REPLACE FUNCTION in_tenancy_v1(
    this_read_tenancy jsonb,
    record_to_check   components_with_attributes
)
RETURNS bool
LANGUAGE sql
IMMUTABLE PARALLEL SAFE CALLED ON NULL INPUT
AS $$
    SELECT in_tenancy_v1(
        this_read_tenancy,
        record_to_check.tenancy_billing_account_pks,
        record_to_check.tenancy_organization_pks,
        record_to_check.tenancy_workspace_pks
    )
$$;

CREATE OR REPLACE FUNCTION is_visible_v1(
    this_visibility jsonb,
    record_to_check components_with_attributes
)
RETURNS bool
LANGUAGE sql
IMMUTABLE PARALLEL SAFE CALLED ON NULL INPUT
AS $$
    SELECT is_visible_v1(
        this_visibility,
        record_to_check.visibility_change_set_pk,
        record_to_check.visibility_deleted_at
    )
$$;

CREATE OR REPLACE FUNCTION in_tenancy_and_visible_v1(
    this_read_tenancy jsonb,
    this_visibility   jsonb,
    record_to_check   components_with_attributes
)
RETURNS bool
LANGUAGE sql
IMMUTABLE PARALLEL SAFE CALLED ON NULL INPUT
AS $$
    SELECT
        in_tenancy_v1(
            this_read_tenancy,
            record_to_check.tenancy_billing_account_pks,
            record_to_check.tenancy_organization_pks,
            record_to_check.tenancy_workspace_pks
        )
        AND is_visible_v1(
            this_visibility,
            record_to_check.visibility_change_set_pk,
            record_to_check.visibility_deleted_at
        )
$$;
