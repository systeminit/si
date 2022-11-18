SELECT row_to_json(fix_batches.*) AS object
FROM fix_batches_v1($1, $2) AS fix_batches
WHERE fix_batches.completion_status IS NOT NULL
