use std::{
    collections::HashMap,
    ffi::OsString,
    fmt, fs,
    ops::{BitOr, BitOrAssign},
    path::Path,
    sync::{atomic::AtomicU64, Arc},
    time::Duration,
};

use client::{SiFsClient, SiFsClientError};
use fuser::{
    FileType, MountOption, ReplyAttr, ReplyData, ReplyDirectory, ReplyEmpty, ReplyEntry, ReplyOpen,
};
use inode_table::{InodeEntryData, InodeTable, InodeTableError};
use nix::{
    libc::{
        EACCES, EINVAL, ENODATA, ENOENT, ENOSYS, ENOTDIR, O_ACCMODE, O_APPEND, O_RDWR, O_WRONLY,
    },
    unistd::{self, Gid, Uid},
};
use si_frontend_types::{fs::kind_pluralized_to_string, FuncKind};
use thiserror::Error;
use tokio::{
    runtime::{self},
    sync::{mpsc::UnboundedReceiver, RwLock},
};

use crate::{async_wrapper::AsyncFuseWrapper, command::FilesystemCommand};

pub use si_id::WorkspaceId;

mod async_wrapper;
mod client;
mod command;
mod inode_table;

const FILE_HANDLE_READ_BIT: FileHandle = FileHandle::new(1 << 63);
const FILE_HANDLE_WRITE_BIT: FileHandle = FileHandle::new(1 << 62);

const FILE_STR_TS_INDEX: &str = "index.ts";

const TTL: Duration = Duration::from_secs(0);

const THIS_DIR: &str = ".";
const PARENT_DIR: &str = "..";

const DIR_STR_CHANGE_SETS: &str = "change-sets";
const DIR_STR_DEFINITION: &str = "definition";
const DIR_STR_FUNCTIONS: &str = "functions";
const DIR_STR_SCHEMAS: &str = "schemas";

#[derive(Error, Debug)]
pub enum SiFileSystemError {
    #[error("inode entry that should exist was not found: {0}")]
    ExpectedInodeNotFound(Inode),
    #[error("inode {0} is not a directory")]
    InodeNotDirectory(Inode),
    #[error("inode table error: {0}")]
    InodeTable(#[from] InodeTableError),
    #[error("si-fs client error: {0}")]
    SiFsClient(#[from] SiFsClientError),
    #[error("std io error: {0}")]
    StdIo(#[from] std::io::Error),
}

pub type SiFileSystemResult<T> = Result<T, SiFileSystemError>;

type RawInode = u64;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Inode(RawInode);

impl Inode {
    const ROOT: Inode = Inode::new(1);

    #[inline]
    const fn new(value: RawInode) -> Self {
        Self(value)
    }

    fn as_raw(&self) -> RawInode {
        self.0
    }
}

impl From<RawInode> for Inode {
    fn from(value: RawInode) -> Self {
        Self::new(value)
    }
}

impl BitOr for Inode {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::from(self.0 | rhs.0)
    }
}

impl BitOrAssign for Inode {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl fmt::Display for Inode {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

type RawFileHandle = u64;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct FileHandle(RawFileHandle);

impl FileHandle {
    #[inline]
    const fn new(value: RawFileHandle) -> Self {
        Self(value)
    }

    fn as_raw(&self) -> RawFileHandle {
        self.0
    }
}

impl From<RawFileHandle> for FileHandle {
    fn from(value: RawFileHandle) -> Self {
        Self::new(value)
    }
}

impl BitOr for FileHandle {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::from(self.0 | rhs.0)
    }
}

impl BitOrAssign for FileHandle {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl fmt::Display for FileHandle {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct OpenFile {
    ino: Inode,
    fh: FileHandle,
    read_buf: Option<Vec<u8>>,
    write_buf: Option<Vec<u8>>,
    append: bool,
    write: bool,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct SiFileSystem {
    client: Arc<SiFsClient>,
    workspace_id: WorkspaceId,
    inode_table: Arc<RwLock<InodeTable>>,
    open_files: Arc<RwLock<HashMap<FileHandle, OpenFile>>>,
    fh_sequence: Arc<AtomicU64>,
    uid: Uid,
    gid: Gid,
}

struct DirEntry {
    ino: Inode,
    name: String,
    kind: FileType,
}

struct DirListing {
    entries: Vec<DirEntry>,
}

impl DirListing {
    pub fn new(ino: Inode, parent: Option<Inode>) -> Self {
        let entries = vec![
            DirEntry {
                ino,
                name: THIS_DIR.into(),
                kind: FileType::Directory,
            },
            DirEntry {
                ino: parent.unwrap_or(Inode::ROOT),
                name: PARENT_DIR.into(),
                kind: FileType::Directory,
            },
        ];

        Self { entries }
    }

    pub fn add(&mut self, ino: Inode, name: String, kind: FileType) {
        self.entries.push(DirEntry { ino, name, kind });
    }

    pub fn ino_for_name(&self, name: &str) -> Option<Inode> {
        self.entries
            .iter()
            .find(|entry| entry.name == name)
            .map(|entry| entry.ino)
    }

    pub fn send_reply(&self, reply: &mut ReplyDirectory, offset: i64) {
        for (i, entry) in self.entries.iter().enumerate().skip(offset as usize) {
            if reply.add(entry.ino.as_raw(), (i + 1) as i64, entry.kind, &entry.name) {
                break;
            }
        }
    }
}

impl SiFileSystem {
    fn new(token: String, endpoint: String, workspace_id: WorkspaceId, uid: Uid, gid: Gid) -> Self {
        let inode_table = InodeTable::new(InodeEntryData::WorkspaceRoot { workspace_id }, uid, gid);

        let client = SiFsClient::new(token, workspace_id, endpoint).unwrap();

        Self {
            client: Arc::new(client),
            workspace_id,
            inode_table: Arc::new(RwLock::new(inode_table)),
            open_files: Arc::new(RwLock::new(HashMap::new())),
            fh_sequence: Arc::new(AtomicU64::new(1)),
            uid,
            gid,
        }
    }

    fn next_file_handle(&self) -> FileHandle {
        FileHandle::from(
            self.fh_sequence
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        )
    }

    async fn getattr(
        &self,
        ino: Inode,
        _fh: Option<FileHandle>,
        reply: ReplyAttr,
    ) -> SiFileSystemResult<()> {
        let Some(entry) = self.inode_table.read().await.entry_for_ino(ino).cloned() else {
            reply.error(ENOENT);
            return Ok(());
        };

        reply.attr(&TTL, entry.attrs());

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn setattr(
        &self,
        ino: Inode,
        _mode: Option<u32>,
        _uid: Option<u32>,
        _gid: Option<u32>,
        size: Option<u64>,
        _fh: Option<FileHandle>,
        _flags: Option<u32>,
        reply: ReplyAttr,
    ) -> SiFileSystemResult<()> {
        // look in the write files table for this file handle, if there is one and set attr is zero
        // say "truncate = true"
        if let Some(size) = size {
            // support truncation. ignore other set size requests for now
            if size == 0 {
                if let Some(attrs) = self.inode_table.write().await.set_size(ino, size) {
                    reply.attr(&TTL, &attrs);
                } else {
                    reply.error(ENOENT);
                }
            }
        } else if let Some(entry) = self.inode_table.read().await.entry_for_ino(ino) {
            reply.attr(&TTL, entry.attrs());
        } else {
            reply.error(ENOENT);
        }

        Ok(())
    }

    async fn open(&self, ino: Inode, reply: ReplyOpen, flags: i32) -> SiFileSystemResult<()> {
        let append = flags & O_APPEND != 0;
        // Are we opening with a file access mode including write access?
        let write_access = matches!(flags & O_ACCMODE, O_RDWR | O_WRONLY);
        // cannot detect O_TRUNC here. Instead we get SetAttr with size = 0;
        let mut fh = self.next_file_handle() | FILE_HANDLE_READ_BIT;
        if write_access {
            fh |= FILE_HANDLE_WRITE_BIT;
        }

        self.open_files.write().await.insert(
            fh,
            OpenFile {
                ino,
                fh,
                read_buf: None,
                write_buf: None,
                append,
                write: write_access,
            },
        );

        reply.opened(fh.as_raw(), 0);

        Ok(())
    }

    async fn opendir(&self, _ino: Inode, reply: ReplyOpen, _flags: i32) -> SiFileSystemResult<()> {
        reply.opened((self.next_file_handle() | FILE_HANDLE_READ_BIT).as_raw(), 0);
        Ok(())
    }

    async fn release(
        &self,
        _ino: Inode,
        fh: FileHandle,
        _flags: i32,
        _lock_owner: Option<u64>,
        _flush: bool,
        reply: ReplyEmpty,
    ) -> SiFileSystemResult<()> {
        reply.ok();
        if let Some(open_file) = self.open_files.write().await.remove(&fh) {
            if open_file.write && open_file.write_buf.is_some() {
                dbg!("should flush?");
            }
        }

        Ok(())
    }

    async fn mkdir(
        &self,
        parent: Inode,
        name: OsString,
        _mode: u32,
        _umask: u32,
        reply: ReplyEntry,
    ) -> SiFileSystemResult<()> {
        let name = name.into_string().expect("received non utf8 name");

        let parent_entry = {
            let inode_table = self.inode_table.read().await;
            let Some(parent_entry) = inode_table.entry_for_ino(parent) else {
                reply.error(ENOENT);
                return Ok(());
            };

            parent_entry.to_owned()
        };

        match parent_entry.data() {
            // `/`
            InodeEntryData::WorkspaceRoot { .. } => {
                reply.error(EINVAL);
            }
            // `/change-sets`
            InodeEntryData::ChangeSets => {
                let change_set = self.client.create_change_set(name.to_owned()).await?;

                let attrs = {
                    let mut inode_table = self.inode_table.write().await;
                    let ino = inode_table.upsert_with_parent_ino(
                        parent,
                        &name,
                        InodeEntryData::ChangeSet {
                            change_set_id: change_set.id,
                            name: name.to_owned(),
                        },
                        FileType::Directory,
                        true,
                        None,
                    )?;
                    inode_table.make_attrs(ino, FileType::Directory, true, Some(512))
                };

                reply.entry(&TTL, &attrs, 1);
            }
            // `/change-sets/$change_set_name`
            InodeEntryData::ChangeSet { .. } => {
                reply.error(ENOSYS);
            }
            InodeEntryData::Schemas { .. } => {
                reply.error(ENOSYS);
            }
            InodeEntryData::AssetFunc { .. } => reply.error(EINVAL),
            InodeEntryData::Schema { .. } => {
                reply.error(ENOSYS);
            }
            InodeEntryData::SchemaVariant { .. } => {
                reply.error(ENOSYS);
            }
            InodeEntryData::ChangeSetFunc { .. } => {
                reply.error(ENOSYS);
            }
            InodeEntryData::ChangeSetFuncs { .. } => {
                reply.error(EACCES);
            }
            InodeEntryData::ChangeSetFuncKind { .. } => {
                reply.error(ENOSYS);
            }
            InodeEntryData::FuncCode { .. } => {
                reply.error(EINVAL);
            }
            InodeEntryData::SchemaVariantDefinition { .. } => {
                reply.error(EINVAL);
            }
            InodeEntryData::SchemaVariantFuncs { .. } => {
                reply.error(EINVAL);
            }
            InodeEntryData::SchemaVariantFuncKind { .. } => {
                reply.error(EINVAL);
            }
            InodeEntryData::SchemaVariantFunc { .. } => {
                reply.error(EINVAL);
            }
        }

        Ok(())
    }

    async fn lookup(
        &self,
        parent: Inode,
        name: impl AsRef<Path>,
        reply: ReplyEntry,
    ) -> SiFileSystemResult<()> {
        let Some(parent_path) = self.inode_table.read().await.path_buf_for_ino(parent) else {
            reply.error(ENOENT);
            return Ok(());
        };

        let name = name.as_ref();
        let full_path = parent_path.join(name);
        let maybe_ino = self.inode_table.read().await.ino_for_path(&full_path);
        let entry_ino = match maybe_ino {
            Some(ino) => ino,
            None => {
                let dir_listing = self.upsert_dir_listing(parent).await?;
                let file_name = name.to_str().unwrap_or_default();
                match dir_listing.ino_for_name(file_name) {
                    Some(ino) => ino,
                    None => {
                        reply.error(ENOENT);
                        return Ok(());
                    }
                }
            }
        };

        if let Some(entry) = self.inode_table.read().await.entry_for_ino(entry_ino) {
            reply.entry(&TTL, entry.attrs(), 0);
        } else {
            reply.error(ENOENT);
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn read(
        &self,
        ino: Inode,
        fh: FileHandle,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyData,
    ) -> SiFileSystemResult<()> {
        let Some(entry) = self.inode_table.read().await.entry_for_ino(ino).cloned() else {
            reply.error(ENOENT);
            return Ok(());
        };

        match entry.data() {
            InodeEntryData::FuncCode {
                change_set_id,
                func_id: id,
                ..
            }
            | InodeEntryData::AssetFunc {
                func_id: id,
                change_set_id,
                ..
            } => {
                let open_files_read = self.open_files.read().await;

                match open_files_read
                    .get(&fh)
                    .and_then(|of| of.read_buf.as_deref())
                {
                    // File handle contents is being tracked
                    Some(bytes) => {
                        reply.data(get_read_slice(bytes, offset as usize, size as usize));
                    }
                    // File handle contents are not yet tracked
                    None => {
                        drop(open_files_read);

                        let code = self.client.func_code(*change_set_id, *id).await?;
                        self.inode_table
                            .write()
                            .await
                            .set_size(ino, code.as_bytes().len() as u64);

                        if let Some(open_file) = self.open_files.write().await.get_mut(&fh) {
                            open_file.read_buf = Some(code.as_bytes().to_vec());
                            if let Some(bytes) = open_file.read_buf.as_deref() {
                                reply.data(get_read_slice(bytes, offset as usize, size as usize));
                            }
                        }
                    }
                }
            }
            _ => reply.error(EINVAL),
        }

        Ok(())
    }

    async fn upsert_dir_listing(&self, ino: Inode) -> SiFileSystemResult<DirListing> {
        let entry = self
            .inode_table
            .read()
            .await
            .entry_for_ino(ino)
            .cloned()
            .ok_or(SiFileSystemError::ExpectedInodeNotFound(ino))?;

        // XXX(fnichol): remove--only for dev debugging
        // {
        //     let ino_path = self
        //         .inode_table
        //         .read()
        //         .await
        //         .path_buf_for_ino(ino)
        //         .ok_or(SiFileSystemError::ExpectedInodeNotFound(ino))?;
        //     dbg!(ino_path, entry.data());
        // }

        let mut dirs = DirListing::new(ino, entry.parent);

        match entry.data() {
            // `/`
            InodeEntryData::WorkspaceRoot { .. } => {
                let mut inode_table = self.inode_table.write().await;
                let ino = inode_table.upsert_with_parent_ino(
                    entry.ino,
                    DIR_STR_CHANGE_SETS,
                    InodeEntryData::ChangeSets,
                    FileType::Directory,
                    true,
                    None,
                )?;
                dirs.add(ino, DIR_STR_CHANGE_SETS.into(), FileType::Directory);
            }
            // `/change-sets/`
            InodeEntryData::ChangeSets => {
                let change_sets = self.client.list_change_sets().await?;

                for change_set in change_sets {
                    let mut inode_table = self.inode_table.write().await;

                    let file_name = &change_set.name;
                    let ino = inode_table.upsert_with_parent_ino(
                        ino,
                        file_name,
                        InodeEntryData::ChangeSet {
                            change_set_id: change_set.id,
                            name: file_name.to_owned(),
                        },
                        FileType::Directory,
                        true,
                        None,
                    )?;

                    dirs.add(ino, file_name.to_owned(), FileType::Directory);
                }
            }
            // `/change-sets/$change_set_name/`
            InodeEntryData::ChangeSet { change_set_id, .. } => {
                let mut inode_table = self.inode_table.write().await;
                let functions_ino = inode_table.upsert_with_parent_ino(
                    entry.ino,
                    DIR_STR_FUNCTIONS,
                    InodeEntryData::ChangeSetFuncs {
                        change_set_id: *change_set_id,
                    },
                    FileType::Directory,
                    true,
                    None,
                )?;
                dirs.add(functions_ino, DIR_STR_FUNCTIONS.into(), FileType::Directory);
                let schemas_ino = inode_table.upsert_with_parent_ino(
                    entry.ino,
                    DIR_STR_SCHEMAS,
                    InodeEntryData::Schemas {
                        change_set_id: *change_set_id,
                    },
                    FileType::Directory,
                    true,
                    None,
                )?;
                dirs.add(schemas_ino, DIR_STR_SCHEMAS.into(), FileType::Directory);
            }
            // `/change-sets/$change_set_name/functions/`
            InodeEntryData::ChangeSetFuncs { change_set_id } => {
                for kind in [
                    FuncKind::Action,
                    FuncKind::Attribute,
                    FuncKind::Authentication,
                    FuncKind::CodeGeneration,
                    FuncKind::Management,
                    FuncKind::Qualification,
                ] {
                    let kind_pluralize_str = kind_pluralized_to_string(kind);
                    let mut inode_table = self.inode_table.write().await;

                    let ino = inode_table.upsert_with_parent_ino(
                        entry.ino,
                        &kind_pluralize_str,
                        InodeEntryData::ChangeSetFuncKind {
                            kind,
                            change_set_id: *change_set_id,
                        },
                        FileType::Directory,
                        true,
                        None,
                    )?;
                    dirs.add(ino, kind_pluralize_str, FileType::Directory);
                }
            }
            // `/change-sets/$change_set_name/functions/$func_kind/`
            InodeEntryData::ChangeSetFuncKind {
                kind,
                change_set_id,
            } => {
                let funcs_of_kind = self
                    .client
                    .change_set_funcs_of_kind(*change_set_id, *kind)
                    .await?;
                for func in funcs_of_kind {
                    let mut inode_table = self.inode_table.write().await;

                    let ino = inode_table.upsert_with_parent_ino(
                        entry.ino,
                        &func.name,
                        InodeEntryData::ChangeSetFunc {
                            func_id: func.id,
                            change_set_id: *change_set_id,
                            size: func.code_size,
                        },
                        FileType::Directory,
                        false,
                        None,
                    )?;
                    dirs.add(ino, func.name, FileType::Directory);
                }
            }
            // `/change-sets/$change_set_name/functions/$func_kind/$func_name/`
            InodeEntryData::ChangeSetFunc {
                func_id: id,
                change_set_id,
                size,
            } => {
                let mut inode_table = self.inode_table.write().await;

                let ino = inode_table.upsert_with_parent_ino(
                    entry.ino,
                    FILE_STR_TS_INDEX,
                    InodeEntryData::FuncCode {
                        func_id: *id,
                        change_set_id: *change_set_id,
                    },
                    FileType::RegularFile,
                    false,
                    Some(*size),
                )?;
                dirs.add(ino, FILE_STR_TS_INDEX.into(), FileType::RegularFile);
            }
            // `/change-sets/$change_set_name/schemas/`
            InodeEntryData::Schemas { change_set_id } => {
                let schemas = self
                    .client
                    .schemas(*change_set_id)
                    .await
                    .expect("failed to fetch variants");

                for schema in schemas {
                    let mut inode_table = self.inode_table.write().await;
                    let ino = inode_table.upsert_with_parent_ino(
                        entry.ino,
                        &schema.name,
                        InodeEntryData::Schema {
                            schema_id: schema.id,
                            name: schema.name.clone(),
                            installed: schema.installed,
                            change_set_id: *change_set_id,
                        },
                        FileType::Directory,
                        true,
                        None,
                    )?;
                    dirs.add(ino, schema.name.clone(), FileType::Directory);
                }
            }
            // `/change-sets/$change_set_name/schemas/$schema_name/`
            InodeEntryData::Schema {
                schema_id: id, change_set_id, ..
            } => {
                let variants = self.client.variants(*change_set_id, *id).await?;

                if let Some(unlocked_variant_id) = variants.unlocked {
                    let mut inode_table = self.inode_table.write().await;

                    let ino = inode_table.upsert_with_parent_ino(
                        entry.ino,
                        "unlocked",
                        InodeEntryData::SchemaVariant {
                            schema_variant_id: unlocked_variant_id,
                            schema_id: *id,
                            change_set_id: *change_set_id,
                            unlocked: true,
                        },
                        FileType::Directory,
                        true,
                        None,
                    )?;
                    dirs.add(ino, "unlocked".into(), FileType::Directory);
                }

                if let Some(locked_variant_id) = variants.locked {
                    let mut inode_table = self.inode_table.write().await;

                    let ino = inode_table.upsert_with_parent_ino(
                        entry.ino,
                        "locked",
                        InodeEntryData::SchemaVariant {
                            schema_variant_id: locked_variant_id,
                            schema_id: *id,
                            change_set_id: *change_set_id,
                            unlocked: false,
                        },
                        FileType::Directory,
                        false,
                        None,
                    )?;
                    dirs.add(ino, "locked".into(), FileType::Directory);
                }
            }
            // `/change-sets/$change_set_name/schemas/$schema_name/$locked`
            InodeEntryData::SchemaVariant {
                schema_id,
                schema_variant_id,
                change_set_id,
                unlocked,
            } => {
                let mut inode_table = self.inode_table.write().await;

                let asset_def_ino = inode_table.upsert_with_parent_ino(
                    entry.ino,
                    DIR_STR_DEFINITION,
                    InodeEntryData::SchemaVariantDefinition {
                        schema_variant_id: *schema_variant_id,
                        schema_id: *schema_id,
                        change_set_id: *change_set_id,
                        unlocked: *unlocked,
                    },
                    FileType::Directory,
                    true,
                    None,
                )?;
                dirs.add(asset_def_ino, DIR_STR_DEFINITION.into(), FileType::Directory);

                let functions_ino = inode_table.upsert_with_parent_ino(
                    entry.ino,
                    DIR_STR_FUNCTIONS,
                    InodeEntryData::SchemaVariantFuncs {
                        schema_variant_id: *schema_variant_id,
                        schema_id: *schema_id,
                        change_set_id: *change_set_id,
                        unlocked: *unlocked,
                    },
                    FileType::Directory,
                    true,
                    None,
                )?;
                dirs.add(functions_ino, DIR_STR_FUNCTIONS.into(), FileType::Directory);
            }
            // `/change-sets/$change_set_name/schemas/$schema_name/$locked/asset-definition/`
            InodeEntryData::SchemaVariantDefinition { schema_id, change_set_id, unlocked, .. } => {
                let asset_func = self
                    .client
                    .asset_func_for_variant(*change_set_id, *schema_id, *unlocked)
                    .await?;
                let ino = self.inode_table.write().await.upsert_with_parent_ino(
                    entry.ino,
                    FILE_STR_TS_INDEX,
                    InodeEntryData::AssetFunc {
                        func_id: asset_func.id,
                        change_set_id: *change_set_id,
                        unlocked: *unlocked,
                    },
                    FileType::RegularFile,
                    *unlocked,
                    Some(asset_func.code_size),
                )?;
                dirs.add(ino, FILE_STR_TS_INDEX.into(), FileType::RegularFile);
            }
            // `/change-sets/$change_set_name/schemas/$schema_name/$locked/functions/`
            InodeEntryData::SchemaVariantFuncs {
                schema_id,
                schema_variant_id,
                change_set_id,
                unlocked,
                ..
            } => {
                for kind in [
                    FuncKind::Action,
                    FuncKind::Attribute,
                    FuncKind::Authentication,
                    FuncKind::CodeGeneration,
                    FuncKind::Management,
                    FuncKind::Qualification,
                ] {
                    let kind_pluralize_str = kind_pluralized_to_string(kind);
                    let mut inode_table = self.inode_table.write().await;

                    let ino = inode_table.upsert_with_parent_ino(
                        entry.ino,
                        &kind_pluralize_str,
                        InodeEntryData::SchemaVariantFuncKind {
                            kind,
                            schema_variant_id: *schema_variant_id,
                            schema_id: *schema_id,
                            change_set_id: *change_set_id,
                            unlocked: *unlocked,
                        },
                        FileType::Directory,
                        true,
                        None,
                    )?;
                    dirs.add(ino, kind_pluralize_str, FileType::Directory);
                }
            }
            // `/change-sets/$change_set_name/schemas/$schema_name/$locked/functions/$func_kind/`
            InodeEntryData::SchemaVariantFuncKind {
                kind,
                schema_id,
                change_set_id,
                unlocked,
                ..
            } => {
                let funcs_of_kind = self
                    .client
                    .variant_funcs_of_kind(*change_set_id, *schema_id, *kind, *unlocked)
                    .await?;
                for func in funcs_of_kind {
                    let mut inode_table = self.inode_table.write().await;

                    let ino = inode_table.upsert_with_parent_ino(
                        entry.ino,
                        &func.name,
                        InodeEntryData::SchemaVariantFunc {
                            func_id: func.id,
                            change_set_id: *change_set_id,
                            size: func.code_size,
                        },
                        FileType::Directory,
                        false,
                        None,
                    )?;
                    dirs.add(ino, func.name, FileType::Directory);
                }
            }
            // `/change-sets/$change_set_name/schemas/$schema_name/$locked/functions/$func_kind/$func_name`
            InodeEntryData::SchemaVariantFunc {
                func_id,
                change_set_id,
                size,
            } => {
                let mut inode_table = self.inode_table.write().await;

                let ino = inode_table.upsert_with_parent_ino(
                    entry.ino,
                    FILE_STR_TS_INDEX,
                    InodeEntryData::FuncCode {
                        func_id: *func_id,
                        change_set_id: *change_set_id,
                    },
                    FileType::RegularFile,
                    false,
                    Some(*size),
                )?;
                dirs.add(ino, FILE_STR_TS_INDEX.into(), FileType::RegularFile);
            }
            // `/change-sets/$change_set_name/functions/$func_kind/$func_name/index.ts`
            InodeEntryData::FuncCode { .. } |
            // `/change-sets/$change_set_name/schemas/$schema_name/$locked/asset-definition/index.ts`
            InodeEntryData::AssetFunc { .. } => {
                // a file is not a directory!
                return Err(SiFileSystemError::InodeNotDirectory(ino));
            }
        }

        Ok(dirs)
    }

    async fn readdir(
        &self,
        ino: Inode,
        _fh: FileHandle,
        offset: i64,
        mut reply: ReplyDirectory,
    ) -> SiFileSystemResult<()> {
        if self.inode_table.read().await.entry_for_ino(ino).is_none() {
            reply.error(ENOENT);
            return Ok(());
        };

        match self.upsert_dir_listing(ino).await {
            Ok(dir_listing) => {
                dir_listing.send_reply(&mut reply, offset);
                reply.ok();
            }
            Err(SiFileSystemError::InodeNotDirectory(_)) => {
                reply.error(ENOTDIR);
            }
            Err(err) => Err(err)?,
        };

        Ok(())
    }

    async fn command_handler_loop(&mut self, mut rx: UnboundedReceiver<FilesystemCommand>) {
        while let Some(command) = rx.recv().await {
            let self_clone = self.clone();
            tokio::task::spawn(async move {
                let res = match command {
                    FilesystemCommand::GetAttr { ino, fh, reply } => {
                        self_clone.getattr(ino, fh, reply).await
                    }
                    FilesystemCommand::ReadDir {
                        ino,
                        fh,
                        offset,
                        reply,
                    } => self_clone.readdir(ino, fh, offset, reply).await,
                    FilesystemCommand::Read {
                        ino,
                        fh,
                        offset,
                        size,
                        flags,
                        lock_owner,
                        reply,
                    } => {
                        self_clone
                            .read(ino, fh, offset, size, flags, lock_owner, reply)
                            .await
                    }
                    FilesystemCommand::Open { reply, ino, flags } => {
                        self_clone.open(ino, reply, flags).await
                    }
                    FilesystemCommand::OpenDir { reply, ino, flags } => {
                        self_clone.opendir(ino, reply, flags).await
                    }
                    FilesystemCommand::Lookup {
                        parent,
                        name,
                        reply,
                    } => self_clone.lookup(parent, name, reply).await,
                    FilesystemCommand::Release {
                        ino,
                        fh,
                        flags,
                        lock_owner,
                        flush,
                        reply,
                    } => {
                        self_clone
                            .release(ino, fh, flags, lock_owner, flush, reply)
                            .await
                    }
                    FilesystemCommand::FSync {
                        ino: _,
                        fh: _,
                        datasync: _,
                        reply,
                    } => {
                        dbg!("fsync!");
                        reply.ok();
                        Ok(())
                    }
                    FilesystemCommand::GetXattr { reply, .. } => {
                        reply.error(ENODATA);
                        Ok(())
                    }
                    FilesystemCommand::SetAttr {
                        ino,
                        mode,
                        uid,
                        gid,
                        size,
                        fh,
                        flags,
                        reply,
                    } => {
                        self_clone
                            .setattr(ino, mode, uid, gid, size, fh, flags, reply)
                            .await
                    }
                    FilesystemCommand::ReleaseDir { reply, .. } => {
                        reply.ok();
                        Ok(())
                    }
                    FilesystemCommand::MkDir {
                        parent,
                        name,
                        mode,
                        umask,
                        reply,
                    } => self_clone.mkdir(parent, name, mode, umask, reply).await,
                    FilesystemCommand::Write {
                        ino: _,
                        fh: _,
                        offset: _,
                        data: _,
                        write_flags: _,
                        flags: _,
                        lock_owner: _,
                        reply,
                    } => {
                        reply.error(ENOSYS);
                        Ok(())
                    }
                    FilesystemCommand::Lseek {
                        ino: _,
                        fh: _,
                        offset: _,
                        whence: _,
                        reply,
                    } => {
                        reply.error(ENOSYS);
                        Ok(())
                    }
                    command => {
                        dbg!(&command);
                        command.error(ENOSYS);
                        Ok(())
                    }
                };

                if let Err(err) = res {
                    dbg!(err);
                }
            });
        }
    }
}

pub fn mount(
    token: String,
    endpoint: String,
    workspace_id: WorkspaceId,
    mount_point: impl AsRef<Path>,
    runtime_handle: runtime::Handle,
    options: Option<Vec<MountOption>>,
) -> SiFileSystemResult<()> {
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let async_fuse_wrapper = AsyncFuseWrapper::new(cmd_tx);

    let uid = unistd::geteuid();
    let gid = unistd::getegid();

    runtime_handle.spawn(async move {
        SiFileSystem::new(token, endpoint, workspace_id, uid, gid)
            .command_handler_loop(cmd_rx)
            .await
    });

    let default_options = vec![
        MountOption::FSName("si-filesystem".to_string()),
        MountOption::NoExec,
        MountOption::RW,
        MountOption::DefaultPermissions,
    ];

    let mut options = options.unwrap_or_default();

    options.extend_from_slice(&default_options);

    let mount_point = mount_point.as_ref();
    fs::create_dir_all(mount_point)?;
    fuser::mount2(async_fuse_wrapper, mount_point, &options)?;

    Ok(())
}

fn get_read_slice(buf: &[u8], offset: usize, size: usize) -> &[u8] {
    let read_len = std::cmp::min(size, buf.len().saturating_sub(offset));
    let read_end = offset.saturating_add(read_len);
    match buf.get(offset..read_end) {
        Some(buf) => buf,
        None => {
            // if this none is hit, it will likely produce an Input/output
            // error, but it should also never be hit :)
            buf
        }
    }
}
