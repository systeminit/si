WITH RECURSIVE filtered_modules AS (
    SELECT
        id,
        name,
        description,
        owner_user_id,
        owner_display_name,
        jsonb_build_object('metadata', metadata) AS metadata,
        latest_hash,
        latest_hash_created_at,
        created_at,
        rejected_at,
        rejected_by_display_name,
        kind,
        is_builtin_at,
        is_builtin_at_by_display_name,
        schema_id
    FROM
        modules 
    WHERE 
        rejected_at IS NULL
        AND kind = 'module'
        AND schema_id IS NOT NULL
        AND latest_hash = ANY($1)
    UNION
    SELECT
        a.id,
        a.name,
        a.description,
        a.owner_user_id,
        a.owner_display_name,
        jsonb_build_object('metadata', a.metadata) AS metadata,
        a.latest_hash,
        a.latest_hash_created_at,
        a.created_at,
        a.rejected_at,
        a.rejected_by_display_name,
        a.kind,
        a.is_builtin_at,
        a.is_builtin_at_by_display_name,
        a.schema_id
    FROM
        modules a
    JOIN
        filtered_modules b
    ON
        a.schema_id = b.schema_id
)
SELECT DISTINCT ON (filtered_modules.schema_id)
    filtered_modules.id,
    filtered_modules.name,
    filtered_modules.description,
    filtered_modules.owner_user_id,
    filtered_modules.owner_display_name,
    filtered_modules.metadata,
    filtered_modules.latest_hash,
    filtered_modules.latest_hash_created_at,
    filtered_modules.created_at,
    filtered_modules.rejected_at,
    filtered_modules.rejected_by_display_name,
    filtered_modules.kind,
    filtered_modules.is_builtin_at,
    filtered_modules.is_builtin_at_by_display_name,
    filtered_modules.schema_id
FROM
    filtered_modules
WHERE
    filtered_modules.is_builtin_at IS NOT NULL -- NOTE(nick,paul): only show the latest, _promoted_ builtin
ORDER BY
    filtered_modules.schema_id,
    filtered_modules.created_at DESC;
