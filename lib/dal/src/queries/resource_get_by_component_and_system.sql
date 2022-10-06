SELECT DISTINCT ON (resources.id) resources.id,
                                  resources.visibility_change_set_pk,
                                  resources.visibility_deleted_at,
                                  row_to_json(resources.*) AS object

FROM resources
WHERE in_tenancy_and_visible_v1($1, $2, resources)
  AND resources.component_id = $3
  AND resources.system_id = $4

ORDER BY resources.id,
         resources.visibility_change_set_pk DESC,
         resources.visibility_deleted_at DESC NULLS FIRST;
