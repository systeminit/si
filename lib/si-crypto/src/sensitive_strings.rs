//! Working with redacting sensitive substrings from content and output.

use std::collections::HashSet;

use si_std::SensitiveString;

const REDACTED_TXT: &str = "[redacted]";

/// Tracks a set of "sensitive" substrings that will be redacted when passed arbitrary strings.
#[derive(Clone, Debug, Default)]
pub struct SensitiveStrings(HashSet<SensitiveString>);

impl SensitiveStrings {
    /// Creates a new empty [`SensitiveStrings`].
    pub fn new(values: HashSet<SensitiveString>) -> Self {
        values.into()
    }

    /// Inserts a new "sensitive" substring, which will be redacted when using [`redact`].
    pub fn insert(&mut self, value: impl Into<SensitiveString>) {
        self.0.insert(value.into());
    }

    /// Inserts multiple "sensitive" substrings from an iterator.
    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = SensitiveString>,
    {
        self.0.extend(iter)
    }

    /// Returns whether or not the given string contains at least one sensitive substring.
    pub fn has_sensitive(&self, s: &str) -> bool {
        self.0
            .iter()
            // Let's not redact empty strings
            .filter(|sensitive_s| !sensitive_s.as_str().is_empty())
            .any(|sensitive_s| s.contains(sensitive_s.as_str()))
    }

    /// Builds a new string with any "sensitive" substrings redacted.
    #[must_use]
    pub fn redact(&self, s: &str) -> String {
        let mut redacted = s.to_string();

        for redacted_str in self.0.iter() {
            // Don't redact empty strings, since this would just add [redacted] after every char
            if redacted_str.as_str().trim().is_empty() {
                continue;
            }

            // Note: This brings a possibility of random substrings being matched out of context,
            // exposing that we have a secret by censoring it but trying to infer word boundary
            // might leak the plaintext credential which is arguably worse
            if s.contains(redacted_str.as_str()) {
                redacted = redacted.replace(redacted_str.as_str(), REDACTED_TXT);
            }
        }

        redacted
    }
}

impl From<HashSet<SensitiveString>> for SensitiveStrings {
    fn from(value: HashSet<SensitiveString>) -> Self {
        Self(value)
    }
}

impl From<SensitiveStrings> for HashSet<SensitiveString> {
    fn from(value: SensitiveStrings) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_sensitive_with_empty() {
        let sensitive_strings = SensitiveStrings::default();

        assert!(!sensitive_strings.has_sensitive("nope"));
    }

    #[test]
    fn has_sensitive_with_empty_string() {
        let mut sensitive_strings = SensitiveStrings::default();
        sensitive_strings.insert("not contained");
        sensitive_strings.insert("");

        assert!(!sensitive_strings.has_sensitive("nope"));
    }

    #[test]
    fn has_sensitive_single_match() {
        let mut sensitive_strings = SensitiveStrings::default();
        sensitive_strings.insert("careful");

        assert!(sensitive_strings.has_sensitive("I should be more careful in the future."));
    }

    #[test]
    fn has_sensitive_multiple_matches() {
        let mut sensitive_strings = SensitiveStrings::default();
        sensitive_strings.insert("careful");
        sensitive_strings.insert("more");

        assert!(sensitive_strings.has_sensitive("I should be more careful in the future."));
    }

    #[test]
    fn redact_with_empty() {
        let sensitive_strings = SensitiveStrings::default();

        assert_eq!(
            "nothing changed",
            sensitive_strings.redact("nothing changed")
        );
    }

    #[test]
    fn redact_with_empty_string() {
        let mut sensitive_strings = SensitiveStrings::default();
        sensitive_strings.insert("not contained");
        sensitive_strings.insert("");

        assert_eq!(
            "nothing changed",
            sensitive_strings.redact("nothing changed")
        );
    }

    #[test]
    fn redact_single_match() {
        let mut sensitive_strings = SensitiveStrings::default();
        sensitive_strings.insert("careful");
        sensitive_strings.insert("pony");

        assert_eq!(
            "I should be more [redacted] in the future.",
            sensitive_strings.redact("I should be more careful in the future.")
        );
    }

    #[test]
    fn redact_multiple_matches() {
        let mut sensitive_strings = SensitiveStrings::default();
        sensitive_strings.insert("apple");
        sensitive_strings.insert("pony");

        assert_eq!(
            "One [redacted] said to the other [redacted]: 'I have an [redacted].'",
            sensitive_strings.redact("One pony said to the other pony: 'I have an apple.'")
        );
    }
}
