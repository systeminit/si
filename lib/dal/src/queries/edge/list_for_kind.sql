SELECT row_to_json(edges.*) AS object
FROM edges_v1($1, $2) as edges
WHERE edges.kind = $3;