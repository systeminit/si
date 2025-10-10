use std::{
    borrow::Cow,
    ffi::{
        OsStr,
        OsString,
    },
    fmt,
    io,
    path::{
        Path,
        PathBuf,
    },
    str::FromStr,
};

use serde_with::{
    DeserializeFromStr,
    SerializeDisplay,
};
use thiserror::Error;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum CanonicalFileError {
    #[error("failed to canonicalize: {1}")]
    Canonicalize(#[source] io::Error, String),
    #[error("file not found: {0}")]
    FileNotFound(String),
    #[error("no file_name after canonicalized join: {0}")]
    NoFileNameAfterJoin(String),
    // needed only for the test
    #[error("var error: {0}")]
    VarError(#[from] std::env::VarError),
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

impl TryFrom<&Path> for CanonicalFile {
    type Error = CanonicalFileError;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
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
    let path_buf = path.canonicalize().map_err(|err| {
        CanonicalFileError::Canonicalize(err, os_str.as_ref().to_string_lossy().to_string())
    })?;
    if !path_buf.is_file() && !path_buf.is_dir() {
        return Err(CanonicalFileError::FileNotFound(
            path_buf.to_string_lossy().to_string(),
        ));
    }

    Ok(path_buf)
}

/// Join a path to a directory safely, in a way that prevents accessing files above the `dir_path`
/// with specially crafted filenames. Assumes `dir_path` is trusted and `file_name` is untrusted
/// input. Does not confirm existence of path, but will fail if the path cannot be canonicalized
/// because the directory does not exist.
pub fn safe_canonically_join(
    dir_path: &Path,
    file_name: impl AsRef<OsStr>,
) -> Result<PathBuf, CanonicalFileError> {
    let full_path = dir_path.join(file_name.as_ref());
    let canonicalized = full_path.canonicalize().map_err(|err| {
        CanonicalFileError::Canonicalize(err, full_path.as_os_str().to_string_lossy().to_string())
    })?;

    match canonicalized.file_name() {
        None => Err(CanonicalFileError::NoFileNameAfterJoin(
            full_path.as_os_str().to_string_lossy().to_string(),
        )),
        Some(file_name) => Ok(dir_path.join(file_name)),
    }
}

#[allow(clippy::panic_in_result_fn)]
#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    // TODO(fnichol): we need to de-CARGO_MANIFEST_DIR this test--better to make a fixture-style
    // directory setup and test that. With Buck2 it's harder to guarentee where we are and if
    // source files are present...
    #[ignore]
    #[test]
    fn test_safe_canonically_join() -> Result<(), CanonicalFileError> {
        // TODO(fnichol): see above, likely we should not be using an environment variable for this
        // test
        #[allow(clippy::disallowed_methods)]
        let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;

        let test_data = vec![
            (
                (&manifest_dir, "../../Cargo.toml"),
                format!("{}/Cargo.toml", &manifest_dir),
            ),
            (
                (&manifest_dir, "../../../../../../../../../etc/passwd"),
                format!("{}/passwd", &manifest_dir),
            ),
            ((&manifest_dir, "../"), format!("{}/lib", &manifest_dir)),
        ];

        for ((dir_path, file_name), expectation) in test_data {
            let joined = safe_canonically_join(Path::new(dir_path), file_name)
                .map(|pathbuf| pathbuf.as_os_str().to_string_lossy().to_string())?;

            assert_eq!(expectation, joined);
        }

        Ok(())
    }
}
