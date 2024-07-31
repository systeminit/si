#[macro_export]
macro_rules! metric {
    ($counter:expr, $value:expr) => {
        info!(metrics = true, "{} = {}", $counter, $value);
    };
}
