use std::{
    borrow::Cow,
    env,
    ffi::{OsStr, OsString},
    io,
    path::{Path, PathBuf},
    str::FromStr,
};

use thiserror::Error;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum CanonicalCommandError {
    #[error("failed to canonicalize: {1}")]
    Canonicalize(#[source] io::Error, PathBuf),
    #[error("program not found on PATH: {0}")]
    NotFound(String),
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CanonicalCommand(PathBuf);

impl CanonicalCommand {
    #[must_use]
    pub fn as_path(&self) -> &Path {
        self.0.as_path()
    }
}

impl AsRef<Path> for CanonicalCommand {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

impl AsRef<OsStr> for CanonicalCommand {
    fn as_ref(&self) -> &OsStr {
        self.0.as_os_str()
    }
}

impl From<CanonicalCommand> for PathBuf {
    fn from(value: CanonicalCommand) -> Self {
        value.0
    }
}

impl TryFrom<Box<Path>> for CanonicalCommand {
    type Error = CanonicalCommandError;

    fn try_from(value: Box<Path>) -> Result<Self, Self::Error> {
        let command = canonicalize_command(value.as_ref())?;
        Ok(Self(command))
    }
}

impl TryFrom<PathBuf> for CanonicalCommand {
    type Error = CanonicalCommandError;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let command = canonicalize_command(value)?;
        Ok(Self(command))
    }
}

impl TryFrom<OsString> for CanonicalCommand {
    type Error = CanonicalCommandError;

    fn try_from(value: OsString) -> Result<Self, Self::Error> {
        let command = canonicalize_command(value)?;
        Ok(Self(command))
    }
}

impl TryFrom<String> for CanonicalCommand {
    type Error = CanonicalCommandError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let command = canonicalize_command(value)?;
        Ok(Self(command))
    }
}

impl TryFrom<&str> for CanonicalCommand {
    type Error = CanonicalCommandError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let command = canonicalize_command(value)?;
        Ok(Self(command))
    }
}

impl<'a> TryFrom<Cow<'a, Path>> for CanonicalCommand {
    type Error = CanonicalCommandError;

    fn try_from(value: Cow<'a, Path>) -> Result<Self, Self::Error> {
        let command = canonicalize_command(value.as_ref())?;
        Ok(Self(command))
    }
}

impl FromStr for CanonicalCommand {
    type Err = CanonicalCommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let command = canonicalize_command(s)?;
        Ok(Self(command))
    }
}

fn canonicalize_command(program: impl AsRef<OsStr>) -> Result<PathBuf, CanonicalCommandError> {
    let found = find_command(program)?;
    found
        .canonicalize()
        .map_err(|err| CanonicalCommandError::Canonicalize(err, found.clone()))
}

fn find_command(program: impl AsRef<OsStr>) -> Result<PathBuf, CanonicalCommandError> {
    let path = Path::new(program.as_ref());

    if path.is_file() {
        Ok(path.to_path_buf())
    } else {
        #[allow(clippy::disallowed_methods)] // We use `$PATH` lookup to find the command
        env::split_paths(&env::var("PATH").unwrap_or_else(|_| "".to_string()))
            .map(|path| path.join(program.as_ref()))
            .find(|candidate| candidate.is_file())
            .ok_or_else(|| {
                CanonicalCommandError::NotFound(program.as_ref().to_string_lossy().to_string())
            })
    }
}
