SELECT row_to_json(actions.*) AS object
FROM actions_v1($1, $2) AS actions
WHERE actions.change_set_pk = $3
ORDER BY index ASC
