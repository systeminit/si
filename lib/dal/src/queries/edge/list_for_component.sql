SELECT row_to_json(edges.*) AS object
FROM edges_v1($1, $2) AS edges
WHERE (head_object_id = $3 OR tail_object_id = $3)
