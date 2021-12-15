use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    fmt, io,
    path::{Path, PathBuf},
    str::FromStr,
};

use serde_with::{DeserializeFromStr, SerializeDisplay};
use telemetry::prelude::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CanonicalFileError {
    #[error("failed to canonicalize: {1}")]
    Canonicalize(#[source] io::Error, String),
    #[error("file not found: {0}")]
    FileNotFound(String),
}

#[derive(
    Clone,
    DeserializeFromStr,
    Debug,
    Default,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    SerializeDisplay,
)]
pub struct CanonicalFile(PathBuf);

impl CanonicalFile {
    #[must_use]
    pub fn as_path(&self) -> &Path {
        self.0.as_path()
    }
}

impl fmt::Display for CanonicalFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.to_string_lossy().as_ref())
    }
}

impl AsRef<Path> for CanonicalFile {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

impl AsRef<OsStr> for CanonicalFile {
    fn as_ref(&self) -> &OsStr {
        self.0.as_os_str()
    }
}

impl From<CanonicalFile> for PathBuf {
    fn from(value: CanonicalFile) -> Self {
        value.0
    }
}

impl TryFrom<Box<Path>> for CanonicalFile {
    type Error = CanonicalFileError;

    fn try_from(value: Box<Path>) -> Result<Self, Self::Error> {
        let command = canonicalize_path(value.as_ref())?;
        Ok(Self(command))
    }
}

impl TryFrom<PathBuf> for CanonicalFile {
    type Error = CanonicalFileError;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let command = canonicalize_path(value)?;
        Ok(Self(command))
    }
}

impl TryFrom<OsString> for CanonicalFile {
    type Error = CanonicalFileError;

    fn try_from(value: OsString) -> Result<Self, Self::Error> {
        let command = canonicalize_path(value)?;
        Ok(Self(command))
    }
}

impl TryFrom<String> for CanonicalFile {
    type Error = CanonicalFileError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let command = canonicalize_path(value)?;
        Ok(Self(command))
    }
}

impl TryFrom<&str> for CanonicalFile {
    type Error = CanonicalFileError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let command = canonicalize_path(value)?;
        Ok(Self(command))
    }
}

impl<'a> TryFrom<Cow<'a, Path>> for CanonicalFile {
    type Error = CanonicalFileError;

    fn try_from(value: Cow<'a, Path>) -> Result<Self, Self::Error> {
        let command = canonicalize_path(value.as_ref())?;
        Ok(Self(command))
    }
}

impl FromStr for CanonicalFile {
    type Err = CanonicalFileError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let command = canonicalize_path(s)?;
        Ok(Self(command))
    }
}

fn canonicalize_path(os_str: impl AsRef<OsStr>) -> Result<PathBuf, CanonicalFileError> {
    let path = Path::new(&os_str);
    // if path.is_relative() {
    // }
    let path_buf = path.canonicalize().map_err(|err| {
        CanonicalFileError::Canonicalize(err, os_str.as_ref().to_string_lossy().to_string())
    })?;
    trace!(path = path_buf.to_string_lossy().as_ref());
    if !path_buf.is_file() {
        return Err(CanonicalFileError::FileNotFound(
            path_buf.to_string_lossy().to_string(),
        ));
    }

    Ok(path_buf)
}
