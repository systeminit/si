SELECT row_to_json(props.*) AS object
FROM props_v1($1, $2) AS props
WHERE props.id IN (
    WITH RECURSIVE recursive_props AS (
        SELECT root_prop_id AS prop_id
        FROM schema_variants_v1($1, $2) AS schema_variants
        WHERE schema_variants.id = $3
        UNION ALL
        SELECT pbp.object_id AS prop_id
        FROM prop_belongs_to_prop_v1($1, $2) AS pbp
        JOIN recursive_props ON pbp.belongs_to_id = recursive_props.prop_id
    )
    SELECT prop_id
    FROM recursive_props
)
