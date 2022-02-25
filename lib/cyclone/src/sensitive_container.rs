use serde::{Deserialize, Serialize};
use std::{fmt, ops::Deref, ops::DerefMut};

// Note: Should this file be here or in si-data? (and make cyclone depend on si-data too)

pub trait ListSecrets {
    fn list_secrets(&self) -> Vec<SensitiveString>;
}

// FIXME: for now ser/de for MaybeSensitive<SensitiveString> is broken
pub type SensitiveString = SensitiveContainer<String>;

#[derive(Debug, Clone, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Copy)]
#[serde(tag = "maybe_sensitive_container_kind")]
pub enum MaybeSensitive<T> {
    Sensitive(SensitiveContainer<T>),
    Plain(T),
}

impl<T: fmt::Display> fmt::Display for MaybeSensitive<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sensitive(container) => container.fmt(f),
            Self::Plain(value) => value.fmt(f),
        }
    }
}

/// A display/debug redacting [`T`].
///
/// The [`SensitiveContainer`] type is wrapper around the `T` type, except that it will
/// not emit its value in its [`std::fmt::Display`] and [`std::fmt::Debug`] implementations. This
/// should be suitable to use when handling passwords, credentials, tokens, etc. as any
/// logging/tracing/debugging should redact it actual value and prevent accidental leaking of
/// sensitive data.
#[derive(Clone, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Copy)]
#[repr(transparent)]
pub struct SensitiveContainer<T>(T);

impl<T> DerefMut for SensitiveContainer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Deref for SensitiveContainer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: fmt::Debug> fmt::Debug for SensitiveContainer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple(&format!(
            "SensitiveContainer<{}>",
            std::any::type_name::<T>()
        ))
        .field(&"...")
        .finish()
    }
}

impl<T: fmt::Display> fmt::Display for SensitiveContainer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("...")
    }
}

impl<T> From<T> for SensitiveContainer<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}
