SELECT DISTINCT ON (schema_id)
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
    AND is_builtin_at IS NOT NULL
ORDER BY
    schema_id,
    created_at DESC;
