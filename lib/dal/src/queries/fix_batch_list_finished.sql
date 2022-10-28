SELECT DISTINCT ON (fix_batches.id) fix_batches.id,
                                    fix_batches.visibility_change_set_pk,
                                    fix_batches.visibility_deleted_at,
                                    row_to_json(fix_batches.*) AS object

FROM fix_batches
WHERE fix_batches.completion_status IS NOT NULL
  AND in_tenancy_and_visible_v1($1, $2, fix_batches)

ORDER BY fix_batches.id,
         fix_batches.visibility_change_set_pk DESC,
         fix_batches.visibility_deleted_at DESC NULLS FIRST
