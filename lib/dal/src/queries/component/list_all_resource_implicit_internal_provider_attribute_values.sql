SELECT row_to_json(attribute_values.*) AS object
FROM attribute_values_v1($1, $2) as attribute_values
INNER JOIN components_v1($1, $2) as components
    ON components.id = attribute_values.attribute_context_component_id
       AND (components.visibility_deleted_at IS NULL OR components.needs_destroy)
LEFT JOIN internal_providers_v1($1, $2) as internal_providers
    ON attribute_values.attribute_context_internal_provider_id = internal_providers.id
LEFT JOIN props_v1($1, $2) as props
    ON props.id = internal_providers.prop_id
       AND props.name = 'resource'
JOIN prop_belongs_to_prop_v1($1, $2) AS prop_belongs_to_prop
    ON prop_belongs_to_prop.object_id = props.id
       AND prop_belongs_to_prop.belongs_to_id IN (
           SELECT prop_many_to_many_schema_variants.left_object_id AS root_prop_id
           FROM prop_many_to_many_schema_variants_v1($1, $2) AS prop_many_to_many_schema_variants
           LEFT JOIN component_belongs_to_schema_variant_v1($1, $2) as component_belongs_to_schema_variant
               ON component_belongs_to_schema_variant.belongs_to_id = prop_many_to_many_schema_variants.right_object_id
       )
