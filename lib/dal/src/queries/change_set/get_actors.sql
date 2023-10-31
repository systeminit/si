select distinct u.email
FROM history_events ha
    INNER JOIN users u ON u.pk = (ha.actor->>'User')::ident
WHERE ha.tenancy_workspace_pk = $1
    AND ha.data::jsonb -> 'visibility' ->> 'visibility_change_set_pk' = $2


