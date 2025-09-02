#![warn(
    bad_style,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

pub mod action;
pub mod cached_default_variant;
pub mod cached_schema;
pub mod cached_schema_variant;
pub mod cached_schemas;
mod change_set;
pub mod checksum;
pub mod component;
pub mod definition_checksum;
pub mod dependent_values;
pub mod incoming_connections;
pub mod index;
pub mod luminork_default_variant;
pub mod luminork_schema_variant;
pub mod management;
pub mod materialized_view;
pub mod object;
pub mod prop_schema;
pub mod reference;
pub mod schema_variant;
pub mod secret;
pub mod temporary_conversion_impls;
pub mod view;

pub use crate::{
    cached_schema::CachedSchema,
    cached_schema_variant::CachedSchemaVariant,
    materialized_view::MaterializedView,
    prop_schema::PropSchemaV1,
    schema_variant::{
        SchemaVariant,
        UninstalledVariant,
    },
    view::{
        View,
        ViewList,
    },
};

#[cfg(test)]
mod tests {
    use serde::{
        Deserialize,
        Serialize,
    };

    use crate::checksum::FrontendChecksum;

    #[test]
    fn enum_with_tuple_variant_bytestreams() {
        #[derive(
            Clone,
            Debug,
            Deserialize,
            Eq,
            PartialEq,
            Ord,
            PartialOrd,
            Serialize,
            si_frontend_mv_types_macros::FrontendChecksum,
        )]
        #[remain::sorted]
        #[serde(rename_all = "camelCase")]
        enum Todd {
            Howard(Vec<u8>, Vec<u8>),
        }

        let item_one = "oblivion remastered";
        let item_two = "tes vi";

        let todd_one = {
            let mut first_vec = Vec::new();
            first_vec.extend(item_one.as_bytes());
            first_vec.extend(item_two.as_bytes());
            let second_vec = Vec::new();
            let todd = Todd::Howard(first_vec, second_vec);
            FrontendChecksum::checksum(&todd)
        };

        let todd_two = {
            let mut first_vec = Vec::new();
            first_vec.extend(item_one.as_bytes());
            let mut second_vec = Vec::new();
            second_vec.extend(item_two.as_bytes());
            let todd = Todd::Howard(first_vec, second_vec);
            FrontendChecksum::checksum(&todd)
        };

        let todd_three = {
            let first_vec = Vec::new();
            let mut second_vec = Vec::new();
            second_vec.extend(item_one.as_bytes());
            second_vec.extend(item_two.as_bytes());
            let todd = Todd::Howard(first_vec, second_vec);
            FrontendChecksum::checksum(&todd)
        };

        assert_ne!(todd_one, todd_two);
        assert_ne!(todd_one, todd_three);
        assert_ne!(todd_two, todd_three);
    }

    #[test]
    fn struct_with_bytestream_fields() {
        #[derive(
            Clone,
            Debug,
            Deserialize,
            Eq,
            PartialEq,
            Ord,
            PartialOrd,
            Serialize,
            si_frontend_mv_types_macros::FrontendChecksum,
        )]
        #[remain::sorted]
        #[serde(rename_all = "camelCase")]
        struct Todd {
            first: Vec<u8>,
            second: Vec<u8>,
        }

        let item_one = "oblivion remastered";
        let item_two = "tes vi";

        let todd_one = {
            let mut first = Vec::new();
            first.extend(item_one.as_bytes());
            first.extend(item_two.as_bytes());
            let second = Vec::new();
            let todd = Todd { first, second };
            FrontendChecksum::checksum(&todd)
        };

        let todd_two = {
            let mut first = Vec::new();
            first.extend(item_one.as_bytes());
            let mut second = Vec::new();
            second.extend(item_two.as_bytes());
            let todd = Todd { first, second };
            FrontendChecksum::checksum(&todd)
        };

        let todd_three = {
            let first = Vec::new();
            let mut second = Vec::new();
            second.extend(item_one.as_bytes());
            second.extend(item_two.as_bytes());
            let todd = Todd { first, second };
            FrontendChecksum::checksum(&todd)
        };

        assert_ne!(todd_one, todd_two);
        assert_ne!(todd_one, todd_three);
        assert_ne!(todd_two, todd_three);
    }
}
