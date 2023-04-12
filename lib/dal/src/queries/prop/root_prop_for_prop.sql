WITH RECURSIVE recursive_props AS (
    SELECT
        $3::ident AS prop_id,
        0::bigint  AS depth
    UNION ALL
    SELECT
        pbp.belongs_to_id         AS prop_id,
        recursive_props.depth + 1 AS depth
    FROM prop_belongs_to_prop_v1($1, $2) AS pbp
    JOIN recursive_props
        ON pbp.object_id = recursive_props.prop_id
)
SELECT row_to_json(props.*) AS object
FROM props_v1($1, $2) as props
INNER JOIN recursive_props rp
    ON rp.prop_id = props.id
ORDER BY depth DESC LIMIT 1;
