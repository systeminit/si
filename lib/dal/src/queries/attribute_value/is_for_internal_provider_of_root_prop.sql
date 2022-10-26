SELECT
    pbtp.belongs_to_id IS NULL AS is_for_root_prop
FROM internal_providers_v1($1, $2) AS ip
LEFT JOIN prop_belongs_to_prop_v1($1, $2) AS pbtp
    ON ip.id = pbtp.object_id
WHERE ip.id = ($3::jsonb ->> 'attribute_context_internal_provider_id')::bigint
