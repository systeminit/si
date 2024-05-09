use std::{
    convert::Infallible, io, iter, option::IntoIter, path::PathBuf, slice, str::FromStr, vec,
};

use thiserror::Error;

mod find;
#[cfg(feature = "layered")]
mod layered_load;
mod simple_load;

pub use config::ValueKind;
pub use find::find;
#[cfg(feature = "layered")]
pub use layered_load::{layered_load, ConfigMap};
#[cfg(feature = "load-str")]
pub use simple_load::load_from_str;
#[cfg(feature = "load-sync")]
pub use simple_load::{load, load_or_default};
#[cfg(feature = "load-async")]
pub use simple_load::{load_async, load_or_default_async};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ConfigFileError {
    #[cfg(feature = "config")]
    #[error("error when building configuration")]
    Builder(#[from] config::ConfigError),
    #[error("failed to canonicalize path {1}")]
    Canonicalize(#[source] io::Error, PathBuf),
    #[error("could not resolve current directory")]
    CurrentDirectory(#[source] io::Error),
    #[error("config file for {0} not found for environment variable {1}='{2}'")]
    EnvNotFound(String, String, String),
    #[cfg(feature = "load-str")]
    #[error("config file too large to read in memory")]
    FileTooBig(#[from] std::num::TryFromIntError),
    #[error("could not determine user's home directory")]
    HomeDirectory,
    #[cfg(feature = "load-json")]
    #[error("error deserializing json")]
    JsonDeserialize(#[from] serde_json::Error),
    #[cfg(any(feature = "load-sync", feature = "load-async"))]
    #[error("error while reading from file")]
    ReadIO(#[source] io::Error),
    #[cfg(feature = "layered")]
    #[error("failed to determine relative path for {0} from {1}")]
    RelativePath(String, String),
    #[cfg(feature = "load-toml")]
    #[error("error deserializing toml")]
    TomlDeserialize(#[from] serde_toml::de::Error),
    #[error("unknown file format with extension: {0}")]
    UnknownFileFormat(String),
    #[cfg(feature = "load-yaml")]
    #[error("error deserializing yaml")]
    YamlDeserialize(#[from] serde_yaml::Error),
}

impl From<Infallible> for ConfigFileError {
    fn from(_: Infallible) -> Self {
        unreachable!();
    }
}

pub type Result<T> = std::result::Result<T, ConfigFileError>;

#[remain::sorted]
#[derive(Clone, Copy, Debug)]
pub enum FileFormat {
    Custom(&'static str),
    #[cfg(feature = "json")]
    Json,
    #[cfg(feature = "toml")]
    Toml,
    #[cfg(feature = "yaml")]
    Yaml,
}

impl FileFormat {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            #[cfg(feature = "toml")]
            FileFormat::Toml => "toml",
            #[cfg(feature = "json")]
            FileFormat::Json => "json",
            #[cfg(feature = "yaml")]
            FileFormat::Yaml => "yaml",
            FileFormat::Custom(custom) => custom,
        }
    }
}

impl FromStr for FileFormat {
    type Err = ConfigFileError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            #[cfg(feature = "toml")]
            "toml" => Ok(Self::Toml),
            #[cfg(feature = "json")]
            "json" => Ok(Self::Json),
            #[cfg(feature = "yaml")]
            "yaml" | "yml" => Ok(Self::Yaml),
            unknwon => Err(Self::Err::UnknownFileFormat(unknwon.to_string())),
        }
    }
}

pub trait ToFileFormats {
    type Iter: Iterator<Item = FileFormat>;

    fn to_file_formats(&self) -> Result<Self::Iter>;
}

impl ToFileFormats for FileFormat {
    type Iter = IntoIter<FileFormat>;

    fn to_file_formats(&self) -> Result<Self::Iter> {
        Ok(Some(*self).into_iter())
    }
}

impl<'a> ToFileFormats for &'a [FileFormat] {
    type Iter = iter::Cloned<slice::Iter<'a, FileFormat>>;

    fn to_file_formats(&self) -> Result<Self::Iter> {
        Ok(self.iter().cloned())
    }
}

impl ToFileFormats for str {
    type Iter = vec::IntoIter<FileFormat>;

    fn to_file_formats(&self) -> Result<Self::Iter> {
        Ok(vec![self.parse()?].into_iter())
    }
}

impl<T: ToFileFormats + ?Sized> ToFileFormats for &T {
    type Iter = T::Iter;

    fn to_file_formats(&self) -> Result<Self::Iter> {
        (**self).to_file_formats()
    }
}
