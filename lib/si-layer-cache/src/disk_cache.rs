use std::{str::FromStr, sync::Arc};

use redb::{AccessGuard, Database, TableDefinition};
use std::path::PathBuf;
use telemetry::tracing::debug;

use crate::{error::LayerDbResult, LayerDbError};

#[derive(Debug, Clone)]
pub struct RedbTempFile {
    pub tempdir: Arc<tempfile::TempDir>,
    pub file_name: PathBuf,
}

impl RedbTempFile {
    fn new(tempdir: tempfile::TempDir, file_name: PathBuf) -> Self {
        Self {
            tempdir: Arc::new(tempdir),
            file_name,
        }
    }
}

pub fn default_redb_path() -> LayerDbResult<RedbTempFile> {
    let tempdir = tempfile::tempdir()?;
    let file_name = tempdir.path().join("si_test.rdb");
    Ok(RedbTempFile::new(tempdir, file_name))
}

pub fn default_redb_path_for_service(service: impl AsRef<str>) -> PathBuf {
    let service = service.as_ref();
    PathBuf::from_str(&format!("/tmp/layerdb-{service}.rdb"))
        .expect("paths from strings is infallible")
}

#[derive(Clone, Debug)]
pub struct DiskCache {
    db: Arc<Database>,
    table_name: Arc<String>,
}

impl DiskCache {
    pub fn new(db: Arc<Database>, table_name: impl Into<String>) -> LayerDbResult<Self> {
        let table_name = table_name.into();
        Ok(Self {
            db,
            table_name: Arc::new(table_name),
        })
    }

    pub async fn get(&self, key: Arc<str>) -> LayerDbResult<Option<AccessGuard<'_, Vec<u8>>>> {
        let db = self.db.clone();
        let table_name = self.table_name.clone();
        let handle =
            tokio::task::spawn_blocking(move || DiskCache::get_blocking(db, table_name, key));
        let result = handle.await?;
        match result {
            Ok(r) => Ok(r),
            Err(LayerDbError::RedbTable(redb::TableError::TableDoesNotExist(t))) => {
                debug!(table = ?t, "get failed because table does not yet exist; this probably means there has been no insert yet.");
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    pub fn get_blocking(
        db: Arc<Database>,
        table_name: Arc<String>,
        key: Arc<str>,
    ) -> LayerDbResult<Option<AccessGuard<'static, Vec<u8>>>> {
        let table_def: TableDefinition<&str, Vec<u8>> = TableDefinition::new(&table_name);
        let read_txn = db.begin_read()?;
        let table = { read_txn.open_table(table_def)? };
        Ok(table.get(&key[..])?)
    }

    pub async fn contains_key(&self, key: Arc<str>) -> LayerDbResult<bool> {
        let result = self.get(key).await?;
        Ok(result.is_some())
    }

    pub async fn insert(&self, key: Arc<str>, value: Vec<u8>) -> LayerDbResult<()> {
        let db = self.db.clone();
        let table_name = self.table_name.clone();
        let handle = tokio::task::spawn_blocking(move || {
            DiskCache::insert_blocking(db, table_name, key, value)
        });
        handle.await?
    }

    pub fn insert_blocking(
        db: Arc<Database>,
        table_name: Arc<String>,
        key: Arc<str>,
        value: Vec<u8>,
    ) -> LayerDbResult<()> {
        let table_def: TableDefinition<&str, Vec<u8>> = TableDefinition::new(&table_name);
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(table_def)?;
            table.insert(&key[..], value)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub async fn remove(&self, key: Arc<str>) -> LayerDbResult<Option<Vec<u8>>> {
        let db = self.db.clone();
        let table_name = self.table_name.clone();
        let handle =
            tokio::task::spawn_blocking(move || DiskCache::remove_blocking(db, table_name, key));
        handle.await?
    }

    pub fn remove_blocking(
        db: Arc<Database>,
        table_name: Arc<String>,
        key: Arc<str>,
    ) -> LayerDbResult<Option<Vec<u8>>> {
        let table_def: TableDefinition<&str, Vec<u8>> = TableDefinition::new(&table_name);
        let write_txn = db.begin_write()?;
        let old_value = {
            let mut table = write_txn.open_table(table_def)?;
            let result = table
                .remove(&key[..])?
                .map(|v| v.value().to_owned().clone());
            result
        };
        write_txn.commit()?;
        Ok(old_value)
    }
}
