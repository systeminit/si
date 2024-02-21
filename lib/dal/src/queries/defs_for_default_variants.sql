SELECT DISTINCT on (svds.id)
  row_to_json(svds.*) as object
FROM schema_variant_definitions_v1($1, $2) AS svds
WHERE schema_variant_id IS NULL OR schema_variant_id in
  (SELECT default_schema_variant_id FROM schemas_v1($1, $2))
