SELECT row_to_json(reconciliation_prototypes.*) AS object
FROM reconciliation_prototypes_v1($1, $2) AS reconciliation_prototypes 
WHERE (reconciliation_prototypes.component_id = $3 OR reconciliation_prototypes.component_id = ident_nil_v1())
      AND reconciliation_prototypes.schema_variant_id = $4
ORDER BY reconciliation_prototypes.component_id DESC
LIMIT 1;
