use std::sync::Arc;

use std::path::PathBuf;

use crate::error::LayerDbResult;
use crate::event::LayeredEvent;

#[derive(Debug, Clone)]
pub struct CaCacheTempFile {
    pub tempdir: Arc<tempfile::TempDir>,
}

impl CaCacheTempFile {
    fn new(tempdir: tempfile::TempDir) -> Self {
        Self {
            tempdir: Arc::new(tempdir),
        }
    }
}

pub fn default_cacache_path() -> LayerDbResult<CaCacheTempFile> {
    let tempdir = tempfile::tempdir()?;
    Ok(CaCacheTempFile::new(tempdir))
}

#[derive(Clone, Debug)]
pub struct DiskCache {
    write_path: Arc<PathBuf>,
}

impl DiskCache {
    pub fn new(dir: impl Into<PathBuf>, table_name: impl Into<String>) -> LayerDbResult<Self> {
        let dir = dir.into();
        let table_name_string = table_name.into();
        let write_path = dir.join(table_name_string);
        Ok(Self {
            write_path: write_path.into(),
        })
    }

    pub async fn get(&self, key: Arc<str>) -> LayerDbResult<Vec<u8>> {
        let data = cacache::read(self.write_path.as_ref(), key).await?;
        Ok(data)
    }

    pub async fn contains_key(&self, key: Arc<str>) -> LayerDbResult<bool> {
        let result = cacache::metadata(self.write_path.as_ref(), key).await?;
        Ok(result.is_some())
    }

    pub async fn insert(&self, key: Arc<str>, value: Vec<u8>) -> LayerDbResult<()> {
        cacache::write(self.write_path.as_ref(), key, value).await?;
        Ok(())
    }

    pub async fn remove(&self, key: Arc<str>) -> LayerDbResult<()> {
        let maybe_metadata = cacache::metadata(self.write_path.as_ref(), key).await?;
        if let Some(metadata) = maybe_metadata {
            cacache::remove_hash(self.write_path.as_ref(), &metadata.integrity).await?;
        }
        Ok(())
    }

    pub async fn write_to_disk(&self, event: Arc<LayeredEvent>) -> LayerDbResult<()> {
        self.insert(event.payload.key.clone(), event.payload.value.to_vec())
            .await?;
        Ok(())
    }

    pub async fn remove_from_disk(&self, event: Arc<LayeredEvent>) -> LayerDbResult<()> {
        self.remove(event.payload.key.clone()).await?;
        Ok(())
    }
}
