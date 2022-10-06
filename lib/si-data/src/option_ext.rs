/// Extensions to [`Option`].
///
/// Provides behavior currently unstable:
/// - `result_option_inspect` [#91345](https://github.com/rust-lang/rust/issues/91345)
pub trait OptionExt<T> {
    /// Calls the provided closure with a reference to the contained value (if [`Some`]).
    ///
    /// # Examples
    ///
    /// ```
    /// # use si_data::OptionExt;
    /// let v = vec![1, 2, 3, 4, 5];
    ///
    /// // prints "got: 4"
    /// let x: Option<&usize> = v.get(3).si_inspect(|x| println!("got: {x}"));
    ///
    /// // prints nothing
    /// let x: Option<&usize> = v.get(5).si_inspect(|x| println!("got: {x}"));
    // ```
    fn si_inspect<F: FnOnce(&T)>(self, f: F) -> Self;

    /// Calls the provided closure if [`None`].
    ///
    /// # Examples
    /// ```
    /// # use si_data::OptionExt;
    /// let v = vec![1, 2, 3, 4, 5];
    ///
    /// // prints nonthing
    /// let x: Option<&usize> = v
    ///     .get(3)
    ///     .si_inspect_none(|| println!("did not find index 3"));
    ///
    /// // prints "did not find index 5"
    /// let x: Option<&usize> = v
    ///     .get(5)
    ///     .si_inspect_none(|| println!("did not find index 5"));
    // ```
    fn si_inspect_none<F: FnOnce()>(self, f: F) -> Self;
}

impl<T> OptionExt<T> for Option<T> {
    fn si_inspect<F: FnOnce(&T)>(self, f: F) -> Self {
        if let Some(ref t) = self {
            f(t);
        }

        self
    }

    fn si_inspect_none<F: FnOnce()>(self, f: F) -> Self {
        if self.is_none() {
            f();
        }

        self
    }
}
