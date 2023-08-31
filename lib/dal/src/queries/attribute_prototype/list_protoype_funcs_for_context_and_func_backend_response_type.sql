SELECT
  row_to_json(funcs.*) AS func_object,
  row_to_json(ap.*) as prototype_object
FROM attribute_prototypes_v1($1, $2) AS ap
JOIN funcs_v1($1, $2) as funcs
  ON funcs.id = ap.func_id
WHERE in_attribute_context_v1($3, ap)
  AND ap.attribute_context_prop_id = $4
  AND funcs.backend_response_type = $5
ORDER BY ap.id
