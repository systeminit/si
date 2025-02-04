use std::{
    collections::BTreeMap,
    io::Cursor,
    path::{Path, PathBuf},
    sync::Arc,
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
    pub name: String,
    pub kind: FileType,
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

    pub fn pending_buf(&self) -> Option<Arc<Cursor<Vec<u8>>>> {
        match self.data() {
            InodeEntryData::SchemaFuncBindingsPending { buf, .. } => Some(buf.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
#[remain::sorted]
pub enum InodeEntryData {
    AssetDefinitionDir {
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
        size: u64,
        attrs_size: u64,
        bindings_size: u64,
        unlocked: bool,
    },
    AssetFuncCode {
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
        unlocked: bool,
    },
    ChangeSet {
        change_set_id: ChangeSetId,
        name: String,
    },
    ChangeSetFuncDir {
        func_id: FuncId,
        change_set_id: ChangeSetId,
        kind: FuncKind,
        size: u64,
    },
    ChangeSetFuncKindDir {
        kind: FuncKind,
        change_set_id: ChangeSetId,
    },
    ChangeSetFuncsDir {
        change_set_id: ChangeSetId,
    },
    ChangeSets,
    FuncCode {
        change_set_id: ChangeSetId,
        func_id: FuncId,
        kind: FuncKind,
    },
    InstalledSchemaMarker,
    SchemaAttrsJson {
        schema_id: SchemaId,
        change_set_id: ChangeSetId,
        unlocked: bool,
    },
    SchemaBindingsJson {
        schema_id: SchemaId,
        change_set_id: ChangeSetId,
        unlocked: bool,
    },
    SchemaDefinitionsDir {
        schema_id: SchemaId,
        change_set_id: ChangeSetId,
    },
    SchemaDir {
        schema_id: SchemaId,
        change_set_id: ChangeSetId,
        name: String,
        installed: bool,
    },
    SchemaFuncBindings {
        change_set_id: ChangeSetId,
        func_id: FuncId,
        kind: FuncKind,
        schema_id: SchemaId,
        size: u64,
        unlocked: bool,
    },
    SchemaFuncBindingsPending {
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
        kind: FuncKind,
        buf: Arc<Cursor<Vec<u8>>>,
        pending_func_id: Option<FuncId>,
    },
    SchemaFuncDir {
        change_set_id: ChangeSetId,
        func_id: FuncId,
        kind: FuncKind,
        schema_id: SchemaId,
        size: u64,
        bindings_size: u64,
        unlocked: bool,
    },
    SchemaFuncKindDir {
        kind: FuncKind,
        schema_id: SchemaId,
        change_set_id: ChangeSetId,
    },
    SchemaFuncsDir {
        schema_id: SchemaId,
        change_set_id: ChangeSetId,
    },
    SchemaFuncVariantsDir {
        kind: FuncKind,
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
        locked_id: Option<FuncId>,
        unlocked_id: Option<FuncId>,
        locked_size: u64,
        locked_bindings_size: u64,
        unlocked_size: u64,
        unlocked_bindings_size: u64,
        pending: bool,
        pending_func_id: Option<FuncId>,
    },
    SchemasDir {
        change_set_id: ChangeSetId,
    },
    WorkspaceRoot {
        workspace_id: WorkspaceId,
    },
}

#[derive(Clone, Debug)]
pub struct InodeTable {
    // This is a vec where the index to the vec is the inode number minus 1
    path_table: Vec<Option<PathBuf>>,
    entries_by_path: BTreeMap<PathBuf, InodeEntry>,
    uid: Uid,
    gid: Gid,
}

pub enum Size {
    Directory,
    UseExisting(u64),
    Force(u64),
}

impl InodeTable {
    pub fn new(
        root_path: impl AsRef<Path>,
        root_entry: InodeEntryData,
        uid: Uid,
        gid: Gid,
    ) -> Self {
        let root_path = root_path.as_ref().to_path_buf();
        let mut table = Self {
            path_table: Vec::with_capacity(4096),
            entries_by_path: BTreeMap::new(),
            uid,
            gid,
        };

        table.upsert(
            root_path,
            root_entry,
            FileType::Directory,
            true,
            Size::Directory,
        );

        table
    }

    pub fn entries_by_path(&self) -> &BTreeMap<PathBuf, InodeEntry> {
        &self.entries_by_path
    }

    pub fn direct_child_entries(&self, ino: Inode) -> InodeTableResult<Vec<InodeEntry>> {
        let mut result = vec![];

        for (_, entry) in self.entries_by_path.iter() {
            if entry.parent != Some(ino) {
                continue;
            }
            result.push(entry.clone());
        }

        Ok(result)
    }

    pub fn path_for_ino(&self, ino: Inode) -> Option<&Path> {
        self.path_table
            .get(ino.as_raw().saturating_sub(1) as usize)
            .and_then(|maybe_p| maybe_p.as_ref().map(|p| p.as_path()))
    }

    pub fn path_buf_for_ino(&self, ino: Inode) -> Option<PathBuf> {
        self.path_table
            .get(ino.as_raw().saturating_sub(1) as usize)
            .cloned()
            .flatten()
    }

    pub fn parent_ino(&self, ino: Inode) -> Option<Inode> {
        self.path_buf_for_ino(ino)
            .and_then(|path_buf| path_buf.parent().map(|path| path.to_path_buf()))
            .and_then(|parent_path| self.entries_by_path.get(&parent_path))
            .map(|entry| entry.ino)
    }

    // pub fn ino_for_path_with_parent(
    //     &self,
    //     parent_ino: Inode,
    //     file_name: impl AsRef<Path>,
    // ) -> InodeTableResult<Option<Inode>> {
    //     let path = self.make_path(Some(parent_ino), file_name)?;
    //     Ok(self.ino_for_path(&path))
    // }
    //

    pub fn ino_for_path(&self, path: &Path) -> Option<Inode> {
        self.entries_by_path.get(path).map(|entry| entry.ino)
    }

    pub fn entry_for_ino(&self, ino: Inode) -> Option<&InodeEntry> {
        self.path_for_ino(ino)
            .and_then(|path| self.entries_by_path.get(path))
    }

    pub fn pending_buf_for_file_with_parent(
        &self,
        parent: Inode,
        file_name: impl AsRef<Path>,
    ) -> InodeTableResult<Option<Arc<Cursor<Vec<u8>>>>> {
        let path = self.make_path(Some(parent), file_name)?;
        Ok(self
            .ino_for_path(&path)
            .and_then(|ino| self.pending_buf_for_ino(ino)))
    }

    pub fn pending_buf_for_ino(&self, ino: Inode) -> Option<Arc<Cursor<Vec<u8>>>> {
        self.path_for_ino(ino)
            .and_then(|path| self.entries_by_path.get(path))
            .and_then(|entry| entry.pending_buf())
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
        size: Size,
    ) -> InodeTableResult<Inode> {
        let path = self.make_path(Some(parent_ino), file_name)?;

        Ok(self.upsert(path, entry_data, kind, write, size))
    }

    pub fn make_attrs(&self, ino: Inode, kind: FileType, write: bool, size: Size) -> FileAttr {
        let perm: u16 = match kind {
            FileType::Directory => if write { 0o755 } else { 0o555 }
            FileType::RegularFile => if write { 0o644 } else  { 0o444 }
            _ => unimplemented!("I don't know why this kind of file was upserted, Only directories and regular files supported"),
        };

        let size = match size {
            Size::Directory => 512,
            Size::UseExisting(fallback) => self
                .entry_for_ino(ino)
                .map(|entry| entry.attrs.size)
                .unwrap_or(fallback),
            Size::Force(forced_size) => forced_size,
        };

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

        self.upsert(ino_path, entry_data, kind, write, Size::Force(size));

        Ok(())
    }

    pub fn invalidate_ino(&mut self, ino: Inode) -> Option<PathBuf> {
        if let Some(parent_path) = match self
            .path_table
            .get_mut(ino.as_raw().saturating_sub(1) as usize)
        {
            Some(entry) => entry.take(),
            None => None,
        } {
            self.entries_by_path.remove(&parent_path);

            // Invalidate all child entries of this inode
            for entry in self
                .path_table
                .iter_mut()
                .skip(ino.as_raw().saturating_sub(1) as usize)
            {
                if let Some(path_buf) = entry {
                    if path_buf.starts_with(&parent_path) {
                        self.entries_by_path.remove(&path_buf.clone());
                        entry.take();
                    }
                }
            }

            Some(parent_path)
        } else {
            None
        }
    }

    pub fn upsert(
        &mut self,
        path: PathBuf,
        entry_data: InodeEntryData,
        kind: FileType,
        write: bool,
        size: Size,
    ) -> Inode {
        let parent = path
            .parent()
            .and_then(|path| self.entries_by_path.get(&path.to_path_buf()))
            .map(|entry| entry.ino);

        let (attrs, ino) = match self.entries_by_path.get(&path) {
            Some(entry) => (self.make_attrs(entry.ino, kind, write, size), entry.ino),
            None => {
                let next_ino = self.next_ino();
                (self.make_attrs(next_ino, kind, write, size), next_ino)
            }
        };

        let file_name = path
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or("".into());

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
                    name: file_name.clone(),
                    data: entry_data.to_owned(),
                    attrs,
                    write,
                    kind,
                }
            })
            .or_insert_with(|| {
                self.path_table.push(Some(path));
                InodeEntry {
                    ino,
                    parent,
                    name: file_name,
                    data: entry_data,
                    attrs,
                    write,
                    kind,
                }
            });

        entry.ino
    }
}
