SELECT row_to_json(ap.*) AS object
FROM attribute_prototypes_v1($1, $2) AS ap
INNER JOIN attribute_prototype_arguments_v1($1, $2) AS apa
    ON apa.attribute_prototype_id = ap.id
WHERE apa.internal_provider_id = $3;
