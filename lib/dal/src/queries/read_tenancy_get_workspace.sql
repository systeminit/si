SELECT DISTINCT ON (workspaces.id) workspaces.id,
                              row_to_json(workspaces.*) AS object
FROM workspaces
WHERE workspaces.id = $1
ORDER BY id DESC
LIMIT 1;
