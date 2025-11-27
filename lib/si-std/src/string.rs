use std::{
    borrow::Cow,
    fmt,
    ops::Deref,
    str::FromStr,
};

use serde::{
    Deserialize,
    Serialize,
};

/// A display/debug redacting [`String`].
///
/// The [`SensitiveString`] type is wrapper around the standard `String` type, except that it will
/// not emit its value in its [`std::fmt::Display`] and [`std::fmt::Debug`] implementations. This
/// should be suitable to use when handling passwords, credentials, tokens, etc. as any
/// logging/tracing/debugging should redact it actual value and prevent accidental leaking of
/// sensitive data.
///
/// When serialized, the value is redacted as "..." to prevent accidental exposure in config dumps
/// or API responses.
#[derive(Clone, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SensitiveString(String);

impl Deref for SensitiveString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Debug for SensitiveString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("...")
    }
}

impl fmt::Display for SensitiveString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("...")
    }
}

impl Serialize for SensitiveString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("...")
    }
}

impl From<String> for SensitiveString {
    /// Converts a `String` into a [`SensitiveString`].
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&String> for SensitiveString {
    /// Converts a `&String` into a [`SensitiveString`].
    ///
    /// This clones `value` and returns the clone.
    fn from(value: &String) -> Self {
        Self(value.clone())
    }
}

impl From<&mut str> for SensitiveString {
    /// Converts a `&mut str` into a [`SensitiveString`].
    ///
    /// The result is allocated on the heap.
    fn from(value: &mut str) -> Self {
        Self(value.to_owned())
    }
}

impl From<&str> for SensitiveString {
    /// Converts a `&str` into a [`String`].
    ///
    /// The result is allocated on the heap.
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<Box<str>> for SensitiveString {
    /// Converts the given boxed `str` slice to a [`SensitiveString`].
    ///
    /// It is notable that the `str` slice is owned.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use si_std::SensitiveString;
    /// let s1: String = String::from("hello world");
    /// let s2: Box<str> = s1.into_boxed_str();
    /// let s3: SensitiveString = SensitiveString::from(s2);
    ///
    /// assert_eq!("hello world", *s3);
    /// ```
    fn from(value: Box<str>) -> Self {
        Self(value.into_string())
    }
}

impl<'a> From<Cow<'a, str>> for SensitiveString {
    /// Converts a clone-on-write string to an owned instance of [`SensitiveString`].
    ///
    /// This extracts the owned string, and clones the string if it is not already owned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use si_std::SensitiveString;
    /// use std::borrow::Cow;
    /// // If the string is not owned
    /// let cow: Cow<str> = Cow::Borrowed("eggplant");
    /// // It will allocate on the heap and copy the string.
    /// let owned: SensitiveString = SensitiveString::from(cow);
    /// assert_eq!(&owned[..], "eggplant");
    /// ```
    fn from(value: Cow<'a, str>) -> Self {
        Self(value.into_owned())
    }
}

impl From<SensitiveString> for String {
    /// Converts a [`SensitiveString`] into a `String.`
    fn from(value: SensitiveString) -> Self {
        value.0
    }
}

impl FromStr for SensitiveString {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() {
        let s = SensitiveString::from("secret");
        // SensitiveString now serializes as "..." to prevent leaking sensitive data
        let serialized = serde_json::to_string(&s).expect("failed to serialize");

        assert_eq!(r#""...""#, serialized);
    }

    #[test]
    fn deserialize() {
        // This will be a JSON string, meaning it includes quotes
        let raw = r#""secret""#;
        let s: SensitiveString = serde_json::from_str(raw).expect("failed to deserialize");

        assert_eq!("secret", s.deref());
    }
}
