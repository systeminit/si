SELECT DISTINCT ON (
    schema_variant_id, 
    component_id
)
    find_schema_variant_id_for_prop_v1(
        $1, $2, ap.attribute_context_prop_id
    ) as schema_variant_id,
    ap.attribute_context_component_id as component_id  
FROM attribute_prototypes_v1($1, $2) as ap
WHERE ap.func_id = $3
ORDER by  
    schema_variant_id,
    component_id 

