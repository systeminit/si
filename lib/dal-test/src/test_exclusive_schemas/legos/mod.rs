pub(crate) use large::{
    migrate_test_exclusive_schema_large_even_lego,
    migrate_test_exclusive_schema_large_odd_lego,
};
pub(crate) use medium::{
    migrate_test_exclusive_schema_medium_even_lego,
    migrate_test_exclusive_schema_medium_odd_lego,
};
pub(crate) use small::{
    migrate_test_exclusive_schema_small_even_lego,
    migrate_test_exclusive_schema_small_odd_lego,
};

mod bricks;
mod large;
mod medium;
mod small;
