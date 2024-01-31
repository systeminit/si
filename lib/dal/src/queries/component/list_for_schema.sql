SELECT row_to_json(c.*) AS object
FROM components_v1($1, $2) AS c
INNER JOIN component_belongs_to_schema_v1($1, $2) AS cbtsv
    ON cbtsv.object_id = c.id
WHERE cbtsv.belongs_to_id = $3;
