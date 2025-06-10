#[macro_export]
macro_rules! getter {
    ($column:ident, $value_type:ty) => {
        pub fn $column(&self) -> &$value_type {
            &self.$column
        }
    };
}

#[macro_export]
macro_rules! getter_copy {
    ($column:ident, $value_type:ty) => {
        pub fn $column(&self) -> $value_type {
            self.$column
        }
    };
}
