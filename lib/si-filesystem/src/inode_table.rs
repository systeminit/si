use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use fuser::{FileAttr, FileType};
use nix::unistd::{Gid, Uid};
use thiserror::Error;

use si_frontend_types::FuncKind;
use si_id::{ChangeSetId, FuncId, SchemaId, SchemaVariantId, WorkspaceId};

#[derive(Error, Debug)]
pub enum InodeTableError {
    #[error("Parent ino {0} not found")]
    ParentInodeNotFound(u64),
}

pub type InodeTableResult<T> = Result<T, InodeTableError>;

#[derive(Clone, Debug)]
pub struct InodeEntry {
    pub ino: u64,
    pub parent: Option<u64>,
    data: InodeEntryData,
    attrs: FileAttr,
}

impl InodeEntry {
    pub fn data(&self) -> &InodeEntryData {
        &self.data
    }

    pub fn attrs(&self) -> &FileAttr {
        &self.attrs
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
#[remain::sorted]
pub enum InodeEntryData {
    ChangeSet {
        id: ChangeSetId,
        name: String,
    },
    ChangeSetFunc {
        id: FuncId,
        change_set_id: ChangeSetId,
        size: u64,
    },
    ChangeSetFuncKind {
        kind: FuncKind,
        change_set_id: ChangeSetId,
    },
    ChangeSetFuncs {
        change_set_id: ChangeSetId,
    },
    FuncCode {
        change_set_id: ChangeSetId,
        id: FuncId,
    },
    Schema {
        id: SchemaId,
        change_set_id: ChangeSetId,
        name: String,
        installed: bool,
    },
    SchemaVariant {
        id: SchemaVariantId,
        schema_id: SchemaId,
        change_set_id: ChangeSetId,
        locked: bool,
    },
    WorkspaceRoot {
        workspace_id: WorkspaceId,
    },
}

#[derive(Clone, Debug)]
pub struct InodeTable {
    // This is a vec where the index to the vec is the inode number minus 1
    path_table: Vec<PathBuf>,
    entries_by_path: HashMap<PathBuf, InodeEntry>,
    uid: Uid,
    gid: Gid,
}

impl InodeTable {
    pub fn new(root_entry: InodeEntryData, uid: Uid, gid: Gid) -> Self {
        let mut table = Self {
            path_table: Vec::with_capacity(4096),
            entries_by_path: HashMap::with_capacity(4096),
            uid,
            gid,
        };

        table.upsert("/".into(), root_entry, FileType::Directory, true, None);

        table
    }

    pub fn path(&self, ino: u64) -> Option<&PathBuf> {
        self.path_table.get(ino.saturating_sub(1) as usize)
    }

    pub fn ino_for_path(&self, path: &PathBuf) -> Option<u64> {
        self.entries_by_path.get(path).map(|entry| entry.ino)
    }

    pub fn get(&self, ino: u64) -> Option<&InodeEntry> {
        self.path(ino)
            .and_then(|path| self.entries_by_path.get(path))
    }

    pub fn next_ino(&self) -> u64 {
        self.path_table.len().saturating_add(1) as u64
    }

    pub fn make_path(
        &self,
        parent: Option<u64>,
        file_name: impl AsRef<Path>,
    ) -> InodeTableResult<PathBuf> {
        Ok(match parent {
            None => "/".into(),
            Some(parent_ino) => self
                .path(parent_ino)
                .map(|parent_path| parent_path.join(file_name))
                .ok_or(InodeTableError::ParentInodeNotFound(parent_ino))?,
        })
    }

    pub fn upsert_with_parent_ino(
        &mut self,
        parent_ino: u64,
        file_name: impl AsRef<Path>,
        entry_data: InodeEntryData,
        kind: FileType,
        write: bool,
        size: Option<u64>,
    ) -> InodeTableResult<u64> {
        let path = self.make_path(Some(parent_ino), file_name)?;

        Ok(self.upsert(path, entry_data, kind, write, size))
    }

    pub fn make_attrs(&self, ino: u64, kind: FileType, perm: u16, size: u64) -> FileAttr {
        FileAttr {
            ino,
            size,
            blocks: 1,
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind,
            perm,
            nlink: 2,
            uid: self.uid.into(),
            gid: self.gid.into(),
            rdev: 0,
            blksize: 512,
            flags: 0,
        }
    }

    pub fn upsert(
        &mut self,
        path: PathBuf,
        entry_data: InodeEntryData,
        kind: FileType,
        write: bool,
        size: Option<u64>,
    ) -> u64 {
        let size = size.unwrap_or(512);
        let parent = path
            .parent()
            .and_then(|path| self.entries_by_path.get(&path.to_path_buf()))
            .map(|entry| entry.ino);

        let next_ino = self.next_ino();

        let perm: u16 = match kind {
            FileType::Directory => if write { 0o755 } else { 0o555 }
            FileType::RegularFile => if write { 0o644 } else  { 0o444 }
            _ => unimplemented!("I don't know why this kind of file was upserted, Only directories and regular files supported"),
        };

        let attrs = self.make_attrs(next_ino, kind, perm, size);

        let entry = self
            .entries_by_path
            .entry(path.clone())
            .and_modify(|entry| {
                let ino = entry.ino;
                let mut attrs = attrs;
                attrs.ino = ino;
                *entry = InodeEntry {
                    ino,
                    parent,
                    data: entry_data.to_owned(),
                    attrs,
                }
            })
            .or_insert_with(|| {
                self.path_table.push(path);
                InodeEntry {
                    ino: next_ino,
                    parent,
                    data: entry_data,
                    attrs,
                }
            });

        entry.ino
    }
}
