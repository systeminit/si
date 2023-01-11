SELECT row_to_json(external_providers.*) AS object
FROM external_providers_v1($1, $2) AS external_providers
WHERE
    schema_variant_id = $3
    AND name = $4
