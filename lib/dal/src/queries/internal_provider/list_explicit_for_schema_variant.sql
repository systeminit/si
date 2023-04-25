SELECT row_to_json(ip.*) AS object
FROM internal_providers_v1($1, $2) AS ip
WHERE schema_variant_id = $3
  AND prop_id = ident_nil_v1();
