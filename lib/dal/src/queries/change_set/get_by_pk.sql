SELECT row_to_json(change_sets) AS object
FROM change_sets
WHERE
    change_sets.pk = $2
    AND in_tenancy_v1($1, change_sets.tenancy_workspace_pks)
