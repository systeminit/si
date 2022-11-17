SELECT DISTINCT ON (workflow_runners.id) workflow_runners.id,
                                                workflow_runners.visibility_change_set_pk,

                                                workflow_runners.component_id,
                                                workflow_runners.schema_id,
                                                workflow_runners.schema_variant_id,
                                                row_to_json(workflow_runners.*) as object
FROM workflow_runners_v1($1, $2) as workflow_runners
WHERE workflow_runners.workflow_prototype_id = $3
  AND (workflow_runners.component_id = $4
       OR workflow_runners.schema_id = $5
       OR workflow_runners.schema_variant_id = $6)
ORDER BY workflow_runners.id,
         visibility_change_set_pk DESC,
         component_id DESC,
         schema_variant_id DESC,
         schema_id DESC;
