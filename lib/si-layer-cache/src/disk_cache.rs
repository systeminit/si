use si_std::CanonicalFile;
use sled::Db;

use crate::error::LayerDbResult;

pub fn default_sled_path() -> LayerDbResult<CanonicalFile> {
    Ok(tempfile::tempdir()?.into_path().try_into()?)
}

#[derive(Clone, Debug)]
pub struct DiskCache {
    tree: sled::Tree,
}

impl DiskCache {
    pub fn new(sled_db: Db, tree_name: impl AsRef<[u8]>) -> LayerDbResult<Self> {
        let tree = sled_db.open_tree(tree_name.as_ref())?;
        Ok(Self { tree })
    }

    pub fn get(&self, key: &str) -> LayerDbResult<Option<Vec<u8>>> {
        Ok(self.tree.get(key.as_bytes())?.map(|bytes| bytes.to_vec()))
    }

    pub fn contains_key(&self, key: &str) -> LayerDbResult<bool> {
        Ok(self.tree.contains_key(key.as_bytes())?)
    }

    pub fn insert(&self, key: &str, value: &[u8]) -> LayerDbResult<()> {
        self.tree.insert(key.as_bytes(), value)?;
        Ok(())
    }

    pub fn remove(&self, key: &str) -> LayerDbResult<Option<Vec<u8>>> {
        let removed_value = self.tree.remove(key.as_bytes())?;
        Ok(removed_value.map(|v| v.to_vec()))
    }
}
