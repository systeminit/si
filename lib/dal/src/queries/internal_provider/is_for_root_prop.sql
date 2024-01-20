SELECT 1 FROM schema_variants_v1($1, $2) AS schema_variants
    JOIN internal_providers_v1($1, $2) AS internal_providers
        ON schema_variants.root_prop_id = internal_providers.prop_id
        AND internal_providers.id = $3;