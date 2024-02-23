SELECT attribute_value_id,
       array_agg(dependent_attribute_value_id) AS dependent_attribute_value_ids
FROM attribute_value_dependency_graph_v1($1, $2, $3)
GROUP BY attribute_value_id;
