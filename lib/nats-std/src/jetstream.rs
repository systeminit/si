//! NATS Jetstream helper functions.

/// Creates a Jetstream stream name that may be prefixed.
pub fn prefixed(prefix: Option<&str>, suffix: impl AsRef<str>) -> String {
    let suffix = suffix.as_ref();

    match prefix {
        Some(prefix) => {
            let mut s = String::with_capacity(prefix.len() + 1 + suffix.len());
            s.push_str(prefix);
            s.push('_');
            s.push_str(suffix);
            s
        }
        None => suffix.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_with_no_prefix() {
        let prefix = None;

        let suffix = "pop_corn";
        let expected = "pop_corn";

        let actual = prefixed(prefix, suffix);

        assert_eq!(expected, actual);
    }

    #[test]
    fn name_with_prefix() {
        let prefix = Some("test_case_123");

        let suffix = "pop_corn";
        let expected = "test_case_123_pop_corn";

        let actual = prefixed(prefix, suffix);

        assert_eq!(expected, actual);
    }
}
