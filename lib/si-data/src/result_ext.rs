/// Extensions to [`Result`].
///
/// Provides behavior currently unstable:
/// - `result_option_inspect` [#91345](https://github.com/rust-lang/rust/issues/91345)
pub trait ResultExt<T, E> {
    /// Calls the provided closure with a reference to the contained value (if [`Ok`]).
    ///
    /// # Examples
    ///
    /// ````
    /// # use si_data::ResultExt;
    /// let x: u8 = "4"
    ///     .parse::<u8>()
    ///     .si_inspect(|x| println!("original: {x}"))
    ///     .map(|x| x.pow(3))
    ///     .expect("failed to parse number");
    /// ````
    fn si_inspect<F: FnOnce(&T)>(self, f: F) -> Self;

    /// Calls the provided closure with a reference to the contained error (if [`Err`]).
    ///
    /// # Examples
    ///
    /// ```
    /// # use si_data::ResultExt;
    /// use std::{fs, io};
    ///
    /// fn read() -> io::Result<String> {
    ///     fs::read_to_string("address.txt")
    ///         .si_inspect_err(|e| eprintln!("failed to read file: {e}"))
    /// }
    /// ```
    fn si_inspect_err<F: FnOnce(&E)>(self, f: F) -> Self;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn si_inspect<F: FnOnce(&T)>(self, f: F) -> Self {
        if let Ok(ref t) = self {
            f(t)
        }

        self
    }

    fn si_inspect_err<F: FnOnce(&E)>(self, f: F) -> Self {
        if let Err(ref e) = self {
            f(e);
        }

        self
    }
}
