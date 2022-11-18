SELECT
    component_id,
    component_name,
    count(qualification_id)                                         as total_qualifications,
    sum(case when qualification_status = 'true' then 1 else 0 end)  as succeeded,
    sum(case when qualification_status = 'false' then 1 else 0 end) as failed
FROM (
    SELECT DISTINCT ON (
        components.component_id,
        qualification_prototypes.id,
        qualification_resolvers.id
    )
        components.component_id,
        components.prop_values -> 'si' ->> 'name'      as component_name,
        qualification_prototypes.id                    as qualification_id,
        func_binding_return_values.value ->> 'success' as qualification_status
    FROM components_with_attributes components
    LEFT JOIN qualification_prototypes_v1($1, $2) AS qualification_prototypes
        ON components.schema_variant_id = qualification_prototypes.schema_variant_id
    LEFT JOIN qualification_resolvers_v1($1, $2) AS qualification_resolvers
        ON components.component_id = qualification_resolvers.component_id
    LEFT JOIN func_binding_return_value_belongs_to_func_binding_v1($1, $2) AS func_binding_return_value_belongs_to_func_binding
        ON func_binding_return_value_belongs_to_func_binding.belongs_to_id =  qualification_resolvers.func_binding_id
    LEFT JOIN func_binding_return_values_v1($1, $2) AS func_binding_return_values
        ON func_binding_return_values.id = func_binding_return_value_belongs_to_func_binding.object_id
    LEFT JOIN component_belongs_to_schema_v1($1, $2) AS component_belongs_to_schema
        ON components.component_id = component_belongs_to_schema.object_id
    LEFT JOIN schemas_v1($1, $2) AS schemas
        ON component_belongs_to_schema.belongs_to_id = schemas.id
    WHERE in_tenancy_and_visible_v1($1, $2, components)
        AND schemas.kind != 'concept'
    ORDER BY
        components.component_id,
        qualification_prototypes.id,
        qualification_resolvers.id,
        qualification_prototypes.visibility_change_set_pk DESC,
        qualification_resolvers.visibility_change_set_pk DESC
) as qualification_data
GROUP BY component_id, component_name
