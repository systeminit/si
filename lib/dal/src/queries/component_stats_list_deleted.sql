SELECT DISTINCT ON (id) id

FROM components

-- Ensure they are deleted
WHERE visibility_deleted_at IS NOT NULL

  -- Compare only to the current change set
  AND visibility_change_set_pk = $2

  -- Check all edit sessions that should contribute to the count
  --   'Open'   specific to you
  --   'Saved'  taken from all edit sessions
  AND visibility_edit_session_pk in (
    SELECT id
    FROM edit_sessions
    WHERE (status = 'Open' AND visibility_edit_session_pk = $3)
       OR status = 'Saved'
        AND in_tenancy_v1($1,
                          tenancy_universal,
                          tenancy_billing_account_ids,
                          tenancy_organization_ids,
                          tenancy_workspace_ids))

  -- Scope the tenancy one last time
  AND in_tenancy_v1($1,
                    tenancy_universal,
                    tenancy_billing_account_ids,
                    tenancy_organization_ids,
                    tenancy_workspace_ids)

ORDER BY id DESC