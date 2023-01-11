SELECT row_to_json(p.*) AS object
FROM props_v1($1, $2) AS p
INNER JOIN attribute_values_v1($1, $2) AS av
    ON av.attribute_context_prop_id = p.id
WHERE av.id = $3;
