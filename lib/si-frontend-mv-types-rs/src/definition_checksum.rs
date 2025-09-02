use std::collections::HashMap;

use si_events::workspace_snapshot::Checksum;

/// Trait for computing definition checksums of types based on their schema/shape.
/// Unlike FrontendChecksum which handles data values, DefinitionChecksum focuses
/// on the structure and type information of the data.
///
/// This trait is used to detect when type schemas change, triggering intelligent
/// MaterializedView rebuilds only when the actual type definition changes,
/// not just the data.
///
/// ## Key Design Principles
///
/// 1. **Type Schema Focus**: Only considers the shape/structure of types,
///    not data values within them
/// 2. **Static Computation**: All checksums computed at compile time using
///    static initialization to avoid runtime overhead
/// 3. **Recursive Handling**: Special support for recursive types using
///    the `#[recursive_definition]` annotation to prevent infinite loops
/// 4. **Type Alias Transparency**: Type aliases produce the same checksum
///    as their underlying type
/// 5. **Compilation Enforcement**: Missing implementations cause compilation
///    errors (no fallback to string representations)
///
/// ## Recursive Type Handling
///
/// For recursive types like `PropSchemaV1`, use the `#[recursive_definition]`
/// annotation on recursive fields:
///
/// ```rust
/// #[derive(DefinitionChecksum)]
/// pub struct PropSchemaV1 {
///     pub prop_id: PropId,
///     pub name: String,
///     // This breaks the recursion cycle while still detecting schema changes
///     #[definition_checksum(recursive_definition)]
///     pub children: Option<Vec<PropSchemaV1>>,
/// }
/// ```
///
/// The recursive annotation uses field name + type string instead of recursively
/// computing the checksum, preventing infinite loops during static initialization
/// while maintaining schema change detection capabilities.
pub trait DefinitionChecksum {
    /// Compute the definition checksum for this type.
    ///
    /// This checksum represents the schema/shape of the type and will change
    /// when the type definition changes (fields added/removed/changed types).
    ///
    /// The checksum is computed statically at compile time and cached in
    /// a LazyLock to avoid runtime computation overhead.
    fn definition_checksum() -> Checksum;
}

#[derive(Debug, Clone)]
pub struct DefinitionChecksumInventoryItem {
    ident: &'static str,
    definition_checksum: &'static ::std::sync::LazyLock<Checksum>,
}

impl DefinitionChecksumInventoryItem {
    pub const fn new(
        ident: &'static str,
        definition_checksum: &'static ::std::sync::LazyLock<Checksum>,
    ) -> Self {
        DefinitionChecksumInventoryItem {
            ident,
            definition_checksum,
        }
    }

    pub fn definition_checksum(&self) -> Checksum {
        **self.definition_checksum
    }

    pub fn ident(&self) -> &'static str {
        self.ident
    }
}

static DEFINITION_CHECKSUMS: ::std::sync::LazyLock<HashMap<String, Checksum>> =
    ::std::sync::LazyLock::new(|| {
        ::inventory::iter::<DefinitionChecksumInventoryItem>()
            .map(|inv_item| (inv_item.ident.to_string(), **inv_item.definition_checksum))
            .collect()
    });

pub fn materialized_view_definition_checksums() -> &'static HashMap<String, Checksum> {
    &DEFINITION_CHECKSUMS
}

::inventory::collect!(DefinitionChecksumInventoryItem);
