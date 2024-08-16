#[macro_export]
macro_rules! metric {
    ($key:ident = $value:expr, *) => {
        info!(metrics = true, $key = $value, *);
    };
    ($key:literal = $value:expr, *) => {
        info!(metrics = true, $key = $value, *);
    };
    ($($key:ident).+ = $value:expr) => {
        info!(metrics = true, $($key).+ = $value);
    };
    ($($key:ident).+ = $value:expr, $label:ident = $label_value:expr) => {
        info!(metrics = true, $($key).+ = $value, $label = $label_value);
    };
}
