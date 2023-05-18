use std::{collections::HashMap, io, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Buck2ResourcesError {
    #[error("failed canonicalize path: `{path}`")]
    Canonicalize { source: io::Error, path: PathBuf },
    #[error("looking for key ending with `{ends_with}` returned multiple matches: {matches:?}")]
    MultipleKeyMatches {
        ends_with: String,
        matches: Vec<String>,
    },
    #[error("failed to look up our own executable path")]
    NoCurrentExe { source: io::Error },
    #[error("executable doesn't have a filename: `{executable_path}`")]
    NoFileName { executable_path: PathBuf },
    #[error("failed to find parent directory of executable: `{executable_path}`")]
    NoParentDir { executable_path: PathBuf },
    #[error("no resource named `{name}` found in manifest file: `{manifest_path}`")]
    NoSuchResource {
        name: String,
        manifest_path: PathBuf,
    },
    #[error("failed to parse manifest file: `{manifest_path}`")]
    ParsingFailed {
        manifest_path: PathBuf,
        source: serde_json::Error,
    },
    #[error("failed to read manifest file: `{manifest_path}`")]
    ReadFailed {
        manifest_path: PathBuf,
        source: io::Error,
    },
}

pub struct Buck2Resources {
    inner: HashMap<String, PathBuf>,
    parent_dir: PathBuf,
    manifest_path: PathBuf,
}

impl Buck2Resources {
    pub fn read() -> Result<Self, Buck2ResourcesError> {
        let executable_path = std::env::current_exe()
            .map_err(|source| Buck2ResourcesError::NoCurrentExe { source })?;
        let parent_dir = match executable_path.parent() {
            Some(p) => p,
            None => return Err(Buck2ResourcesError::NoParentDir { executable_path }),
        };
        let file_name = match executable_path.file_name() {
            Some(f) => f,
            None => return Err(Buck2ResourcesError::NoFileName { executable_path }),
        };
        let manifest_path =
            parent_dir.join(format!("{}.resources.json", file_name.to_string_lossy()));
        let manifest_string = match std::fs::read_to_string(&manifest_path) {
            Ok(s) => s,
            Err(source) => {
                return Err(Buck2ResourcesError::ReadFailed {
                    manifest_path,
                    source,
                })
            }
        };
        let inner: HashMap<String, PathBuf> =
            serde_json::from_str(&manifest_string).map_err(|source| {
                Buck2ResourcesError::ParsingFailed {
                    manifest_path: manifest_path.clone(),
                    source,
                }
            })?;

        Ok(Self {
            inner,
            parent_dir: parent_dir.to_path_buf(),
            manifest_path,
        })
    }

    pub fn get(&self, name: impl AsRef<str>) -> Result<PathBuf, Buck2ResourcesError> {
        let rel_path =
            self.inner
                .get(name.as_ref())
                .ok_or_else(|| Buck2ResourcesError::NoSuchResource {
                    name: name.as_ref().to_string(),
                    manifest_path: self.manifest_path.clone(),
                })?;

        let path = self.parent_dir.join(rel_path);
        let path = path
            .canonicalize()
            .map_err(|source| Buck2ResourcesError::Canonicalize { source, path })?;

        Ok(path)
    }

    pub fn get_ends_with(&self, name: impl AsRef<str>) -> Result<PathBuf, Buck2ResourcesError> {
        let ends_with = format!("/{}", name.as_ref());
        let mut candidates: Vec<_> = self
            .inner
            .keys()
            .filter_map(|key| {
                if key.ends_with(&ends_with) {
                    Some(key.as_str())
                } else {
                    None
                }
            })
            .collect();

        if candidates.is_empty() {
            return Err(Buck2ResourcesError::NoSuchResource {
                name: format!("*{ends_with}"),
                manifest_path: self.manifest_path.clone(),
            });
        }
        if candidates.len() >= 2 {
            return Err(Buck2ResourcesError::MultipleKeyMatches {
                ends_with,
                matches: candidates.into_iter().map(|c| c.to_string()).collect(),
            });
        }

        match candidates.pop() {
            Some(key) => self.get(key),
            None => unreachable!("candidates has len == 1"),
        }
    }
}
