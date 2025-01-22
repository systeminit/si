use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use fuser::{FileAttr, FileType};
use nix::unistd::{Gid, Uid};
use thiserror::Error;

use si_frontend_types::FuncKind;
use si_id::{ChangeSetId, FuncId, SchemaId, WorkspaceId};

use crate::Inode;

#[derive(Error, Debug)]
pub enum InodeTableError {
    #[error("Parent ino {0} not found")]
    ParentInodeNotFound(Inode),
    #[error("ino {0} not found")]
    InodeNotFound(Inode),
}

pub type InodeTableResult<T> = Result<T, InodeTableError>;

#[derive(Clone, Debug)]
pub struct InodeEntry {
    pub ino: Inode,
    pub parent: Option<Inode>,
    kind: FileType,
    pub data: InodeEntryData,
    attrs: FileAttr,
    write: bool,
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
    AssetFunc {
        func_id: FuncId,
        change_set_id: ChangeSetId,
        size: u64,
        unlocked: bool,
    },
    AssetFuncCode {
        func_id: FuncId,
        change_set_id: ChangeSetId,
    },
    ChangeSet {
        change_set_id: ChangeSetId,
        name: String,
    },
    ChangeSetFunc {
        func_id: FuncId,
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
    ChangeSets,
    FuncCode {
        change_set_id: ChangeSetId,
        func_id: FuncId,
    },
    InstalledSchemaMarker,
    Schema {
        schema_id: SchemaId,
        change_set_id: ChangeSetId,
        name: String,
        installed: bool,
    },
    SchemaDefinitions {
        schema_id: SchemaId,
        change_set_id: ChangeSetId,
    },
    SchemaFunc {
        change_set_id: ChangeSetId,
        func_id: FuncId,
        size: u64,
        unlocked: bool,
    },
    SchemaFuncKind {
        kind: FuncKind,
        schema_id: SchemaId,
        change_set_id: ChangeSetId,
    },
    SchemaFuncs {
        schema_id: SchemaId,
        change_set_id: ChangeSetId,
    },
    SchemaFuncVariants {
        locked_id: Option<FuncId>,
        unlocked_id: Option<FuncId>,
        change_set_id: ChangeSetId,
        locked_size: u64,
        unlocked_size: u64,
    },
    Schemas {
        change_set_id: ChangeSetId,
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

    pub fn path_for_ino(&self, ino: Inode) -> Option<&Path> {
        self.path_table
            .get(ino.as_raw().saturating_sub(1) as usize)
            .map(|p| p.as_path())
    }

    pub fn path_buf_for_ino(&self, ino: Inode) -> Option<PathBuf> {
        self.path_table
            .get(ino.as_raw().saturating_sub(1) as usize)
            .cloned()
    }

    pub fn ino_for_path(&self, path: &Path) -> Option<Inode> {
        self.entries_by_path.get(path).map(|entry| entry.ino)
    }

    pub fn entry_for_ino(&self, ino: Inode) -> Option<&InodeEntry> {
        self.path_for_ino(ino)
            .and_then(|path| self.entries_by_path.get(path))
    }

    pub fn entry_mut_for_ino(&mut self, ino: Inode) -> Option<&mut InodeEntry> {
        self.path_buf_for_ino(ino)
            .and_then(|path_buf| self.entries_by_path.get_mut(path_buf.as_path()))
    }

    pub fn next_ino(&self) -> Inode {
        Inode::new(self.path_table.len().saturating_add(1) as u64)
    }

    pub fn make_path(
        &self,
        parent: Option<Inode>,
        file_name: impl AsRef<Path>,
    ) -> InodeTableResult<PathBuf> {
        Ok(match parent {
            None => "/".into(),
            Some(parent_ino) => self
                .path_for_ino(parent_ino)
                .map(|parent_path| parent_path.join(file_name))
                .ok_or(InodeTableError::ParentInodeNotFound(parent_ino))?,
        })
    }

    pub fn upsert_with_parent_ino(
        &mut self,
        parent_ino: Inode,
        file_name: impl AsRef<Path>,
        entry_data: InodeEntryData,
        kind: FileType,
        write: bool,
        size: Option<u64>,
    ) -> InodeTableResult<Inode> {
        let path = self.make_path(Some(parent_ino), file_name)?;

        Ok(self.upsert(path, entry_data, kind, write, size))
    }

    pub fn make_attrs(
        &self,
        ino: Inode,
        kind: FileType,
        write: bool,
        size: Option<u64>,
    ) -> FileAttr {
        let perm: u16 = match kind {
            FileType::Directory => if write { 0o755 } else { 0o555 }
            FileType::RegularFile => if write { 0o644 } else  { 0o444 }
            _ => unimplemented!("I don't know why this kind of file was upserted, Only directories and regular files supported"),
        };
        let size = size.unwrap_or(512);

        FileAttr {
            ino: ino.as_raw(),
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

    pub fn set_size(&mut self, ino: Inode, size: u64) -> Option<FileAttr> {
        self.entry_mut_for_ino(ino).map(|entry| {
            entry.attrs.size = size;
            entry.attrs
        })
    }

    pub fn upsert_for_ino(&mut self, ino: Inode, entry: InodeEntry) -> InodeTableResult<()> {
        let ino_path = self
            .path_buf_for_ino(ino)
            .ok_or(InodeTableError::InodeNotFound(ino))?;

        let size = entry.attrs().size;
        let kind = entry.kind;
        let write = entry.write;
        let entry_data = entry.data;

        self.upsert(ino_path, entry_data, kind, write, Some(size));

        Ok(())
    }

    pub fn upsert(
        &mut self,
        path: PathBuf,
        entry_data: InodeEntryData,
        kind: FileType,
        write: bool,
        size: Option<u64>,
    ) -> Inode {
        let parent = path
            .parent()
            .and_then(|path| self.entries_by_path.get(&path.to_path_buf()))
            .map(|entry| entry.ino);

        let next_ino = self.next_ino();

        let attrs = self.make_attrs(next_ino, kind, write, size);

        let entry = self
            .entries_by_path
            .entry(path.clone())
            .and_modify(|entry| {
                let ino = entry.ino;
                let mut attrs = attrs;
                attrs.ino = ino.as_raw();
                *entry = InodeEntry {
                    ino,
                    parent,
                    data: entry_data.to_owned(),
                    attrs,
                    write,
                    kind,
                }
            })
            .or_insert_with(|| {
                self.path_table.push(path);
                InodeEntry {
                    ino: next_ino,
                    parent,
                    data: entry_data,
                    attrs,
                    write,
                    kind,
                }
            });

        entry.ino
    }
}
