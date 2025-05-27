//! NATS [`Subject`] helper functions.

use si_data_nats::Subject;

/// Creates a [`Subject`] that may be prefixed.
pub fn prefixed(prefix: Option<&str>, suffix: impl AsRef<str>) -> Subject {
    let suffix = suffix.as_ref();
    match prefix {
        Some(prefix) => Subject::from(format!("{prefix}.{suffix}")),
        None => Subject::from(suffix),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_with_no_prefix() {
        let prefix = None;

        let suffix = "pop.corn";
        let expected = Subject::from_static("pop.corn");

        let actual = prefixed(prefix, suffix);

        assert_eq!(expected, actual);
    }

    #[test]
    fn name_with_prefix() {
        let prefix = Some("test.case.123");

        let suffix = "pop.corn";
        let expected = Subject::from_static("test.case.123.pop.corn");

        let actual = prefixed(prefix, suffix);

        assert_eq!(expected, actual);
    }
}
