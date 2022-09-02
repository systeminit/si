SELECT DISTINCT ON (id) row_to_json(internal_providers.*) AS object
FROM internal_providers
WHERE in_tenancy_and_visible_v1($1, $2, internal_providers)
  AND prop_id = $3
ORDER BY id,
         visibility_change_set_pk DESC,
         visibility_deleted_at DESC NULLS FIRST;
