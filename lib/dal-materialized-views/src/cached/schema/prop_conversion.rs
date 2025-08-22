use std::{
    collections::hash_map::DefaultHasher,
    hash::{
        Hash,
        Hasher,
    },
};

use si_frontend_mv_types::prop_schema::PropSchemaV1;
use si_id::{
    PropId,
    SchemaVariantId,
    ulid::{
        self,
        Ulid,
    },
};
use si_pkg::{
    HasUniqueId,
    PropSpec,
};
use telemetry::prelude::*;
use thiserror::Error;

/// Base ULID for generating deterministic PropIds for uninstalled schema variants.
///
/// All uninstalled variants use deterministic PropIds generated from this base
/// since they don't have real PropIds until installation. We use deterministic
/// generation rather than random ULIDs to ensure materialized views have stable,
/// deterministic output - this is critical for MV caching and consistency.
const UNINSTALLED_PROP_ID_BASE: &str = "01K37FBCPPJF80NHSCCDYBA92Z";

/// Generate a deterministic ULID for uninstalled props based on their variant and path.
///
/// This ensures that the same prop path in the same variant always generates the same ULID across
/// multiple MV builds, maintaining MV stability, while ensuring different prop paths or different
/// variants get different ULIDs to avoid conflicts.
fn generate_uninstalled_prop_ulid(
    variant_id: SchemaVariantId,
    prop_path: &str,
) -> Result<Ulid, ConversionError> {
    // Create a unique key by combining variant_id and prop_path
    let unique_key = format!("{variant_id}/{prop_path}");

    // Hash the unique key to get a deterministic number
    let mut hasher = DefaultHasher::new();
    unique_key.hash(&mut hasher);
    let path_hash = hasher.finish();

    // Use the base ULID timestamp + combine with path hash for randomness part
    let base_ulid = Ulid::from_string(UNINSTALLED_PROP_ID_BASE).map_err(|e| {
        ConversionError::InvalidBaseUlid {
            base_ulid: UNINSTALLED_PROP_ID_BASE.to_string(),
            source: e,
        }
    })?;
    let timestamp = base_ulid.timestamp_ms();

    // Create new ULID with same timestamp but path-based randomness
    Ok(Ulid::from_parts(timestamp, path_hash as u128))
}

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error(
        "PropSpec missing required unique_id - schema: '{schema_name}', variant: '{variant_id}', prop_path: '{prop_path}'"
    )]
    MissingUniqueId {
        schema_name: String,
        variant_id: SchemaVariantId,
        prop_path: String,
    },
    #[error(
        "Failed to parse PropSpec unique_id '{unique_id}' as ULID - schema: '{schema_name}', variant: '{variant_id}', prop_path: '{prop_path}': {source}"
    )]
    InvalidUniqueId {
        unique_id: String,
        schema_name: String,
        variant_id: SchemaVariantId,
        prop_path: String,
        #[source]
        source: ulid::DecodeError,
    },
    #[error("Invalid base ULID constant '{base_ulid}': {source}")]
    InvalidBaseUlid {
        base_ulid: String,
        #[source]
        source: ulid::DecodeError,
    },
}

/// Convert a PropSpec tree to PropSchemaV1 tree for cached MVs.
///
/// For uninstalled schema variants, props use deterministic ULIDs generated from
/// their path since they don't have real PropIds until installation. This ensures
/// materialized views have stable, deterministic output which is critical for MV
/// caching and consistency.
pub fn convert_prop_spec_to_schema_v1(
    prop_spec: &PropSpec,
    schema_name: &str,
    variant_id: SchemaVariantId,
    prop_path: &str,
) -> Result<PropSchemaV1, ConversionError> {
    // Generate PropId: use existing unique_id if available, otherwise generate deterministic ULID
    let prop_id = match prop_spec.unique_id() {
        Some(unique_id_str) => {
            // Parse existing unique_id string to PropId
            unique_id_str.parse::<PropId>().map_err(|e| {
                error!(
                    "Failed to parse PropSpec unique_id '{}' as ULID - schema: '{}', variant: '{}', prop_path: '{}': {}",
                    unique_id_str, schema_name, variant_id, prop_path, e
                );
                ConversionError::InvalidUniqueId {
                    unique_id: unique_id_str.to_string(),
                    schema_name: schema_name.to_string(),
                    variant_id,
                    prop_path: prop_path.to_string(),
                    source: e,
                }
            })?
        }
        None => {
            // Generate deterministic ULID for uninstalled variants
            debug!(
                "Generating deterministic prop ID for uninstalled variant - schema: '{}', variant: '{}', prop_path: '{}'",
                schema_name, variant_id, prop_path
            );
            generate_uninstalled_prop_ulid(variant_id, prop_path)?.into()
        }
    };

    let data = prop_spec.data();

    // Convert child PropSpecs based on the PropSpec type
    let children = convert_children_with_context(prop_spec, schema_name, variant_id, prop_path)?;

    Ok(PropSchemaV1 {
        prop_id,
        name: prop_spec.name().to_string(),
        prop_type: convert_prop_spec_kind(prop_spec.kind()),
        description: data.and_then(|d| d.documentation.clone()),
        children: if children.is_empty() {
            None
        } else {
            Some(children)
        },
        // New fields from PropSpecData (excluding func/widget/ui fields)
        validation_format: data.and_then(|d| d.validation_format.clone()),
        default_value: data.and_then(|d| d.default_value.clone()),
        hidden: data.and_then(|d| d.hidden),
        doc_link: data.and_then(|d| d.doc_link.as_ref().map(|u| u.to_string())),
    })
}

/// Convert child PropSpecs based on the parent PropSpec type
fn convert_children_with_context(
    prop_spec: &PropSpec,
    schema_name: &str,
    variant_id: SchemaVariantId,
    parent_path: &str,
) -> Result<Vec<PropSchemaV1>, ConversionError> {
    use si_pkg::PropSpecKind;

    let children = prop_spec.direct_children();
    let mut result = Vec::with_capacity(children.len());

    match prop_spec.kind() {
        PropSpecKind::Object => {
            // Object entries: convert all children
            for child in children {
                let child_path = format!("{}/{}", parent_path, child.name());
                let child_schema =
                    convert_prop_spec_to_schema_v1(child, schema_name, variant_id, &child_path)?;
                result.push(child_schema);
            }
        }
        PropSpecKind::Array | PropSpecKind::Map => {
            // Array/Map: convert single type_prop as child
            if let Some(type_child) = children.first() {
                let child_path = format!("{}/{}", parent_path, type_child.name());
                let child_schema = convert_prop_spec_to_schema_v1(
                    type_child,
                    schema_name,
                    variant_id,
                    &child_path,
                )?;
                result.push(child_schema);
            }
        }
        PropSpecKind::String
        | PropSpecKind::Boolean
        | PropSpecKind::Number
        | PropSpecKind::Float
        | PropSpecKind::Json => {
            // Leaf nodes: no children
        }
    }

    Ok(result)
}

/// Convert PropSpecKind to prop_type string
fn convert_prop_spec_kind(kind: si_pkg::PropSpecKind) -> String {
    use si_pkg::PropSpecKind;

    match kind {
        PropSpecKind::String => "string",
        PropSpecKind::Boolean => "boolean",
        PropSpecKind::Object => "object",
        PropSpecKind::Array => "array",
        PropSpecKind::Map => "map",
        PropSpecKind::Number => "number",
        PropSpecKind::Float => "float",
        PropSpecKind::Json => "json",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use si_pkg::PropSpec;

    use super::*;

    #[test]
    fn test_convert_string_prop() {
        let prop_spec = PropSpec::builder()
            .name("test_string")
            .kind(si_pkg::PropSpecKind::String)
            .unique_id("01234567890123456789012345")
            .documentation("Test documentation")
            .build()
            .expect("Failed to build test PropSpec");

        let result = convert_prop_spec_to_schema_v1(
            &prop_spec,
            "TestSchema",
            SchemaVariantId::generate(),
            "domain/test_string",
        );

        assert!(result.is_ok(), "Conversion should succeed");
        let schema = result.expect("Conversion result should be Ok");
        assert_eq!(schema.name, "test_string");
        assert_eq!(schema.prop_type, "string");
        assert_eq!(schema.description, Some("Test documentation".to_string()));
        assert!(schema.children.is_none());
    }

    #[test]
    fn test_missing_unique_id_generates_deterministic_ulid() {
        let prop_spec = PropSpec::builder()
            .name("test_string")
            .kind(si_pkg::PropSpecKind::String)
            .build()
            .expect("Failed to build test PropSpec");

        let variant_id = SchemaVariantId::generate();
        let result = convert_prop_spec_to_schema_v1(
            &prop_spec,
            "TestSchema",
            variant_id,
            "domain/test_string",
        );

        assert!(result.is_ok(), "Conversion should succeed");
        let schema = result.expect("Conversion result should be Ok");
        assert_eq!(schema.name, "test_string");
        assert_eq!(schema.prop_type, "string");

        // Verify a deterministic ULID was generated
        let expected_prop_id = generate_uninstalled_prop_ulid(variant_id, "domain/test_string")
            .expect("ULID generation should succeed in test");
        assert_eq!(schema.prop_id, expected_prop_id.into());
    }

    #[test]
    fn test_domain_prop_uses_deterministic_ulid() {
        let prop_spec = PropSpec::builder()
            .name("domain")
            .kind(si_pkg::PropSpecKind::Object)
            .build()
            .expect("Failed to build domain PropSpec");

        let variant_id = SchemaVariantId::generate();
        let result = convert_prop_spec_to_schema_v1(&prop_spec, "TestSchema", variant_id, "domain");

        assert!(result.is_ok(), "Conversion should succeed");
        let schema = result.expect("Conversion result should be Ok");
        assert_eq!(schema.name, "domain");
        assert_eq!(schema.prop_type, "object");

        // Verify the deterministic ULID is used and parses correctly
        let expected_prop_id = generate_uninstalled_prop_ulid(variant_id, "domain")
            .expect("ULID generation should succeed in test");
        assert_eq!(schema.prop_id, expected_prop_id.into());
    }

    #[test]
    fn test_different_paths_generate_different_ulids() {
        let prop_spec1 = PropSpec::builder()
            .name("prop1")
            .kind(si_pkg::PropSpecKind::String)
            .build()
            .expect("Failed to build test PropSpec");

        let prop_spec2 = PropSpec::builder()
            .name("prop2")
            .kind(si_pkg::PropSpecKind::String)
            .build()
            .expect("Failed to build test PropSpec");

        let result1 = convert_prop_spec_to_schema_v1(
            &prop_spec1,
            "TestSchema",
            SchemaVariantId::generate(),
            "domain/prop1",
        )
        .expect("Conversion should succeed for prop1");

        let result2 = convert_prop_spec_to_schema_v1(
            &prop_spec2,
            "TestSchema",
            SchemaVariantId::generate(),
            "domain/prop2",
        )
        .expect("Conversion should succeed for prop2");

        // Different paths should generate different ULIDs
        assert_ne!(result1.prop_id, result2.prop_id);
    }

    #[test]
    fn test_base_ulid_is_valid() {
        // Verify our base ULID is actually valid
        let result = UNINSTALLED_PROP_ID_BASE.parse::<PropId>();
        assert!(
            result.is_ok(),
            "Base ULID should be valid: {UNINSTALLED_PROP_ID_BASE}",
        );
    }

    #[test]
    fn test_deterministic_ulid_generation() {
        // Test that the generation function produces valid ULIDs
        let variant_id = SchemaVariantId::generate();
        let ulid1 = generate_uninstalled_prop_ulid(variant_id, "domain/test")
            .expect("ULID generation should succeed in test");
        let ulid2 = generate_uninstalled_prop_ulid(variant_id, "domain/test")
            .expect("ULID generation should succeed in test");
        let ulid3 = generate_uninstalled_prop_ulid(variant_id, "domain/other")
            .expect("ULID generation should succeed in test");

        // Same path should generate same ULID
        assert_eq!(ulid1, ulid2);

        // Different paths should generate different ULIDs
        assert_ne!(ulid1, ulid3);

        // Generated ULIDs should be valid PropIds
        let prop_id1: PropId = ulid1.into();
        let prop_id2: PropId = ulid3.into();
        assert_ne!(prop_id1, prop_id2);
    }

    #[test]
    fn test_different_variants_generate_different_ulids() {
        // Test that the same prop path in different variants generates different ULIDs
        let prop_spec = PropSpec::builder()
            .name("domain")
            .kind(si_pkg::PropSpecKind::Object)
            .build()
            .expect("Failed to build domain PropSpec");

        let variant_id1 = SchemaVariantId::generate();
        let variant_id2 = SchemaVariantId::generate();

        let result1 =
            convert_prop_spec_to_schema_v1(&prop_spec, "TestSchema", variant_id1, "domain")
                .expect("Conversion should succeed for variant1");

        let result2 =
            convert_prop_spec_to_schema_v1(&prop_spec, "TestSchema", variant_id2, "domain")
                .expect("Conversion should succeed for variant2");

        // Different variants should generate different ULIDs for the same prop path
        assert_ne!(result1.prop_id, result2.prop_id);
        assert_eq!(result1.name, result2.name); // Same prop name
        assert_eq!(result1.prop_type, result2.prop_type); // Same prop type
    }

    #[test]
    fn test_mv_stability_same_input_same_output() {
        // Test that the same prop produces identical output (MV stability)
        let prop_spec = PropSpec::builder()
            .name("domain")
            .kind(si_pkg::PropSpecKind::Object)
            .build()
            .expect("Failed to build domain PropSpec");

        let variant_id = SchemaVariantId::generate();

        let result1 =
            convert_prop_spec_to_schema_v1(&prop_spec, "TestSchema", variant_id, "domain")
                .expect("First conversion should succeed");

        let result2 =
            convert_prop_spec_to_schema_v1(&prop_spec, "TestSchema", variant_id, "domain")
                .expect("Second conversion should succeed");

        // Results should be identical for MV stability
        assert_eq!(result1.prop_id, result2.prop_id);
        assert_eq!(result1.name, result2.name);
        assert_eq!(result1.prop_type, result2.prop_type);
    }
}
