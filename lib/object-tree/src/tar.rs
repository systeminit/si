use std::path::{Path, PathBuf};

use crate::Hash;

pub mod read;
pub mod write;

fn object_path(hash: &Hash) -> PathBuf {
    Path::new("objects").join(hash.to_string())
}

fn ref_path(name: impl AsRef<Path>) -> PathBuf {
    Path::new("refs").join(name)
}
