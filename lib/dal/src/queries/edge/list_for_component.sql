SELECT row_to_json(edges.*) AS object
FROM edges_v1($1, $2) AS edges
WHERE (head_component_id = $3 OR tail_component_id = $3)
