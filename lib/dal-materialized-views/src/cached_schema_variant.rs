use dal::{
    DalContext,
    SchemaVariantId,
    cached_module::CachedModule,
};
use si_frontend_mv_types::cached_schema_variant::CachedSchemaVariant as CachedSchemaVariantMv;
use si_id::FuncId;
use si_pkg::HasUniqueId;
use telemetry::prelude::*;

/// Generic helper function to collect function IDs from any function type
/// that implements HasUniqueId
fn collect_function_ids<T, E, I, F>(
    get_functions: F,
    collector: &mut Vec<FuncId>,
) -> Result<(), E>
where
    F: FnOnce() -> Result<I, E>,
    I: IntoIterator<Item = T>,
    T: HasUniqueId,
    E: std::error::Error,
{
    for func in get_functions()? {
        if let Some(unique_id) = func.unique_id() {
            match unique_id.parse::<FuncId>() {
                Ok(func_id) => collector.push(func_id),
                Err(e) => warn!("Failed to parse function ID '{}': {}", unique_id, e),
            }
        }
    }
    Ok(())
}

#[instrument(
    name = "dal_materialized_views.cached_schema_variant",
    level = "debug",
    skip_all
)]
pub async fn assemble(
    ctx: DalContext,
    id: SchemaVariantId,
) -> super::Result<CachedSchemaVariantMv> {
    // Find the cached module containing this variant by storing the module info
    //
    // TODO: We should really look at extracting the variant information into a table, so we
    //       don't have to dig through the data like this.
    for mut module in CachedModule::latest_modules(&ctx).await? {
        let si_pkg = module.si_pkg(&ctx).await?;
        let schemas = si_pkg.schemas()?;

        for schema in schemas {
            let variants = schema.variants()?;
            for variant in variants {
                if let Some(unique_id) = variant.unique_id() {
                    if let Ok(variant_id) = unique_id.parse::<SchemaVariantId>() {
                        if variant_id == id {
                            // Found the variant, process it immediately to avoid lifetime issues
                            let variant_data = variant.data().ok_or_else(|| {
                                super::Error::SchemaVariant(dal::SchemaVariantError::NotFound(id))
                            })?;

                            // Get the asset func unique_id and convert to FuncId
                            let asset_func_id = variant_data
                                .func_unique_id()
                                .parse::<FuncId>()
                                .map_err(|_| {
                                    super::Error::SchemaVariant(dal::SchemaVariantError::NotFound(
                                        id,
                                    ))
                                })?;

                            // Get category from schema data
                            let category = schema
                                .data
                                .as_ref()
                                .map(|d| d.category())
                                .unwrap_or("Component") // default category
                                .to_string();

                            // Get display name from variant spec
                            let variant_spec = variant.to_spec().await?;
                            let display_name = variant_spec
                                .data
                                .as_ref()
                                .and_then(|d| d.display_name.as_ref())
                                .map(|d| d.to_string())
                                .unwrap_or_else(|| schema.name().to_string()); // fallback to schema name

                            // Collect all function IDs attached to this variant
                            let mut variant_func_ids = Vec::with_capacity(100); // Pre-allocate based on typical size

                            // All function types use the same helper with closure syntax
                            collect_function_ids(|| variant.leaf_functions(), &mut variant_func_ids)?;
                            collect_function_ids(|| variant.action_funcs(), &mut variant_func_ids)?;
                            collect_function_ids(|| variant.auth_funcs(), &mut variant_func_ids)?;
                            collect_function_ids(|| variant.management_funcs(), &mut variant_func_ids)?;
                            collect_function_ids(|| variant.si_prop_funcs(), &mut variant_func_ids)?;
                            collect_function_ids(|| variant.root_prop_funcs(), &mut variant_func_ids)?;

                            // Remove duplicates efficiently, and ensure we have stable output for the MV.
                            variant_func_ids.sort_unstable();
                            variant_func_ids.dedup();

                            // Determine if this variant is the default variant for the schema
                            let is_default_variant = if let Some(schema_data) = schema.data.as_ref()
                            {
                                if let Some(default_variant_unique_id) =
                                    schema_data.default_schema_variant()
                                {
                                    // Check if this variant's unique_id matches the default
                                    variant.unique_id() == Some(default_variant_unique_id)
                                } else {
                                    // No default specified in schema, check if this is the first variant
                                    let all_variants = schema.variants()?;
                                    all_variants.first().map(|first| first.unique_id())
                                        == variant.unique_id().map(Some)
                                }
                            } else {
                                // No schema data, assume first variant is default
                                let all_variants = schema.variants()?;
                                all_variants.first().map(|first| first.unique_id())
                                    == variant.unique_id().map(Some)
                            };

                            return Ok(CachedSchemaVariantMv::new(
                                id,
                                display_name,
                                category,
                                variant_data.color().unwrap_or("").to_string(),
                                true, // is_locked - cached modules are locked until installed
                                variant_data.description().map(|d| d.to_string()),
                                variant_data.link().map(|l| l.to_string()),
                                asset_func_id,
                                variant_func_ids,
                                is_default_variant,
                            ));
                        }
                    }
                }
            }
        }
    }

    // Variant not found
    Err(super::Error::SchemaVariant(
        dal::SchemaVariantError::NotFound(id),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use si_id::FuncId;

    // Mock error type for testing
    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    #[error("{0}")]
    struct TestError(&'static str);

    // Mock struct implementing HasUniqueId for testing
    #[derive(Clone)]
    struct MockFunc {
        id: Option<String>,
    }

    impl HasUniqueId for MockFunc {
        fn unique_id(&self) -> Option<&str> {
            self.id.as_deref()
        }
    }

    #[test]
    fn test_collect_function_ids_success() {
        let func_id_1 = FuncId::generate();
        let func_id_2 = FuncId::generate();
        let mock_funcs = vec![
            MockFunc { id: Some(func_id_1.to_string()) },
            MockFunc { id: Some(func_id_2.to_string()) },
            MockFunc { id: None }, // Should be skipped
        ];

        let mut collector = Vec::new();
        let result: Result<(), TestError> = collect_function_ids(|| Ok(mock_funcs), &mut collector);

        assert!(result.is_ok());
        assert_eq!(collector.len(), 2);
    }

    #[test]
    fn test_collect_function_ids_invalid_uuid() {
        let valid_func_id = FuncId::generate();
        let mock_funcs = vec![
            MockFunc { id: Some(valid_func_id.to_string()) },
            MockFunc { id: Some("invalid-uuid".to_string()) }, // Should be logged and skipped
        ];

        let mut collector = Vec::new();
        let result: Result<(), TestError> = collect_function_ids(|| Ok(mock_funcs), &mut collector);

        assert!(result.is_ok());
        assert_eq!(collector.len(), 1); // Only valid one collected
    }

    #[test]
    fn test_collect_function_ids_closure_error_propagation() {
        let mut collector = Vec::new();
        let result = collect_function_ids(|| -> Result<Vec<MockFunc>, TestError> { 
            Err(TestError("Test error"))
        }, &mut collector);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TestError("Test error"));
        assert_eq!(collector.len(), 0);
    }

    #[test]
    fn test_collect_function_ids_empty_collection() {
        let mock_funcs: Vec<MockFunc> = vec![];
        let mut collector = Vec::new();
        let result: Result<(), TestError> = collect_function_ids(|| Ok(mock_funcs), &mut collector);

        assert!(result.is_ok());
        assert_eq!(collector.len(), 0);
    }

    #[test]
    fn test_deduplication_logic() {
        let func_id_1 = FuncId::generate();
        let func_id_2 = FuncId::generate();
        
        let mut func_ids = vec![func_id_1, func_id_2, func_id_1, func_id_2]; // Duplicates
        
        func_ids.sort_unstable();
        func_ids.dedup();
        
        assert_eq!(func_ids.len(), 2);
        assert!(func_ids.contains(&func_id_1));
        assert!(func_ids.contains(&func_id_2));
    }

    #[test]
    fn test_malformed_function_ids_edge_cases() {
        let valid_func_id = FuncId::generate();
        let mock_funcs = vec![
            MockFunc { id: Some("".to_string()) }, // Empty string
            MockFunc { id: Some("not-a-uuid".to_string()) }, // Invalid format
            MockFunc { id: Some("123".to_string()) }, // Too short
            MockFunc { id: Some("x".repeat(100)) }, // Too long
            MockFunc { id: Some(valid_func_id.to_string()) }, // Valid
        ];

        let mut collector = Vec::new();
        let result: Result<(), TestError> = collect_function_ids(|| Ok(mock_funcs), &mut collector);

        // Should not panic, should log warnings, should return Ok with only valid IDs
        assert!(result.is_ok());
        assert_eq!(collector.len(), 1); // Only the valid UUID
    }

    #[test]
    fn test_performance_with_large_collections() {
        use std::time::Instant;

        // Create a large number of mock functions to test performance
        let large_mock_funcs: Vec<MockFunc> = (0..1000)
            .map(|_| MockFunc {
                id: Some(FuncId::generate().to_string()),
            })
            .collect();

        let start = Instant::now();
        let mut collector = Vec::with_capacity(1000);
        let result: Result<(), TestError> = collect_function_ids(|| Ok(large_mock_funcs.clone()), &mut collector);
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert_eq!(collector.len(), 1000);
        
        // Should complete in reasonable time (adjust threshold as needed)
        assert!(duration.as_millis() < 100, "Performance test took too long: {:?}", duration);
    }

    #[test]
    fn test_mixed_valid_invalid_ids() {
        let func_id_1 = FuncId::generate();
        let func_id_2 = FuncId::generate();
        let mock_funcs = vec![
            MockFunc { id: Some(func_id_1.to_string()) }, // Valid
            MockFunc { id: Some("invalid".to_string()) }, // Invalid - should be skipped
            MockFunc { id: Some(func_id_2.to_string()) }, // Valid
            MockFunc { id: None }, // None - should be skipped
        ];

        let mut collector = Vec::new();
        let result: Result<(), TestError> = collect_function_ids(|| Ok(mock_funcs), &mut collector);

        assert!(result.is_ok());
        assert_eq!(collector.len(), 2); // Only valid ones
    }

    // Compilation tests to ensure type constraints work correctly
    #[test]
    fn test_helper_function_type_constraints() {
        // Verify our helper function accepts the right types
        let func_id = FuncId::generate();
        let mock_funcs = vec![MockFunc { id: Some(func_id.to_string()) }];
        let mut collector = Vec::new();
        
        // This should compile - verifies type constraints are correct
        let _result: Result<(), TestError> = collect_function_ids(|| Ok(mock_funcs), &mut collector);
    }
}
