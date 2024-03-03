use std::path::Path;

use sled::{self, IVec};

use crate::{error::LayerCacheResult, CacheType};

#[derive(Debug)]
pub struct DiskCache {
    pub db: sled::Db,
    pub object_tree: sled::Tree,
    pub graph_tree: sled::Tree,
}

impl DiskCache {
    pub fn new(path: impl AsRef<Path>) -> LayerCacheResult<DiskCache> {
        let db = sled::open(path)?;
        let object_tree = db.open_tree([CacheType::Object as u8])?;
        let graph_tree = db.open_tree([CacheType::Graph as u8])?;
        Ok(DiskCache {
            db,
            object_tree,
            graph_tree,
        })
    }

    fn get_tree(&self, cache_type: &CacheType) -> &sled::Tree {
        match cache_type {
            CacheType::Graph => &self.graph_tree,
            CacheType::Object => &self.object_tree,
        }
    }

    pub fn get(
        &self,
        cache_type: &CacheType,
        key: impl AsRef<[u8]>,
    ) -> LayerCacheResult<Option<IVec>> {
        let tree = self.get_tree(cache_type);
        let result = tree.get(key)?;
        Ok(result)
    }

    pub fn contains_key(
        &self,
        cache_type: &CacheType,
        key: impl AsRef<[u8]>,
    ) -> LayerCacheResult<bool> {
        let tree = self.get_tree(cache_type);
        let key = key.as_ref();
        let result = tree.contains_key(key)?;
        Ok(result)
    }

    pub fn insert(
        &self,
        cache_type: &CacheType,
        key: impl AsRef<[u8]>,
        value: impl Into<Vec<u8>>,
    ) -> LayerCacheResult<()> {
        let tree = self.get_tree(cache_type);
        let key = key.as_ref();
        let _result = tree.insert(key, value.into())?;
        Ok(())
    }
}
