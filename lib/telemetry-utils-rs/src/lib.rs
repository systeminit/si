#[macro_export]
macro_rules! metric {
    ($key:ident = $value:expr_2021, *) => {
        info!(metrics = true, $key = $value, *);
    };
    ($key:literal = $value:expr_2021, *) => {
        info!(metrics = true, $key = $value, *);
    };
    ($($key:ident).+ = $value:expr_2021) => {
        info!(metrics = true, $($key).+ = $value);
    };
    ($($key:ident).+ = $value:expr_2021, $label:ident = $label_value:expr_2021) => {
        info!(metrics = true, $($key).+ = $value, $label = $label_value);
    };
}
