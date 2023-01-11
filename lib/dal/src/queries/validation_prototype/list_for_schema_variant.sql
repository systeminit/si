SELECT row_to_json(validation_prototypes.*) AS object
FROM validation_prototypes_v1($1, $2) as validation_prototypes
INNER JOIN props_v1($1, $2) as props
    ON props.id = validation_prototypes.prop_id
    AND props.id IN (
        WITH RECURSIVE recursive_props AS (
            SELECT left_object_id AS prop_id
            FROM prop_many_to_many_schema_variants_v1($1, $2) AS prop_many_to_many_schema_variants
            WHERE right_object_id = $3
            UNION ALL
            SELECT pbp.object_id AS prop_id
            FROM prop_belongs_to_prop_v1($1, $2) AS pbp
            JOIN recursive_props ON pbp.belongs_to_id = recursive_props.prop_id
        )
    SELECT prop_id
    FROM recursive_props
)
