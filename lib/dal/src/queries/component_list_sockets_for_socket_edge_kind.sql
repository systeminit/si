SELECT row_to_json(s.*) AS object
FROM sockets_v1($1, $2) AS s
INNER JOIN socket_many_to_many_schema_variants_v1($1, $2) AS smtmsv
    ON s.id = smtmsv.left_object_id
INNER JOIN component_belongs_to_schema_variant_v1($1, $2) AS cbtsv
    ON smtmsv.right_object_id = cbtsv.belongs_to_id
INNER JOIN components_v1($1, $2) AS c
    ON cbtsv.object_id = c.id
        AND c.id = $3
WHERE s.edge_kind = $4;
