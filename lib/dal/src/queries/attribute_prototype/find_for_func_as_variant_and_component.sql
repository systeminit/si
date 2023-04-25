SELECT DISTINCT ON (
    schema_variant_id, 
    component_id
)
    props.schema_variant_id as schema_variant_id,
    ap.attribute_context_component_id as component_id  
FROM attribute_prototypes_v1($1, $2) as ap
    JOIN props_v1($1, $2) as props
        ON props.id = ap.attribute_context_prop_id
WHERE ap.func_id = $3
ORDER BY
    schema_variant_id,
    component_id 

