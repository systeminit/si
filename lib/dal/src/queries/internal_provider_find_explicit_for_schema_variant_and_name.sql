SELECT row_to_json(ip.*) AS object
FROM internal_providers_v1($1, $2) AS ip
WHERE
    prop_id = ident_nil_v1()
    AND schema_variant_id = $3
    AND name = $4;
