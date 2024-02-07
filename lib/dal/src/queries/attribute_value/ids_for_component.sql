SELECT av.id AS attribute_value_id
FROM attribute_values_v1($1, $2) AS av
    WHERE av.attribute_context_component_id = $3;
