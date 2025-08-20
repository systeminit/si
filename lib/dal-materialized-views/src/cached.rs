use si_id::FuncId;
use si_pkg::HasUniqueId;
use telemetry::prelude::*;

pub mod schema;
pub mod schemas;

/// Collects function IDs from SiPkg function types, handling invalid IDs gracefully.
/// Logs warnings for function IDs that cannot be parsed as ULIDs and skips them.
pub(crate) fn collect_function_ids<T, E, I, F>(
    get_functions: F,
    collector: &mut Vec<FuncId>,
    schema_id: &str,
    variant_id: &str,
    func_type: &str,
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
                Err(e) => {
                    debug!(
                        "Skipping invalid function ID '{}' in {} for schema {} variant {}: {}. Expected ULID format.",
                        unique_id, func_type, schema_id, variant_id, e
                    );
                }
            }
        }
    }
    Ok(())
}
