SELECT DISTINCT ON (workspaces.id) workspaces.id,
                              workspaces.visibility_change_set_pk,
                              workspaces.visibility_edit_session_pk,
                              row_to_json(workspaces.*) AS object
FROM workspaces
WHERE workspaces.id = $1
  AND is_visible_v1($2, workspaces.visibility_change_set_pk, workspaces.visibility_edit_session_pk, workspaces.visibility_deleted)
ORDER BY id DESC, visibility_change_set_pk DESC, visibility_edit_session_pk DESC
LIMIT 1;
