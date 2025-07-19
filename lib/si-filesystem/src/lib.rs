use core::str;
use std::{
    collections::{
        BTreeMap,
        HashMap,
        HashSet,
    },
    ffi::OsString,
    fmt,
    fs,
    io::{
        Cursor,
        Seek,
        Write,
    },
    ops::{
        BitOr,
        BitOrAssign,
    },
    path::{
        Path,
        PathBuf,
    },
    str::Utf8Error,
    sync::{
        Arc,
        atomic::AtomicU64,
    },
    time::Duration,
};

use client::{
    SiFsClient,
    SiFsClientError,
};
use dashmap::DashMap;
use fuser::{
    FileAttr,
    FileType,
    MountOption,
    ReplyAttr,
    ReplyCreate,
    ReplyData,
    ReplyDirectory,
    ReplyEmpty,
    ReplyEntry,
    ReplyOpen,
    ReplyWrite,
};
use inode_table::{
    InodeEntry,
    InodeEntryData,
    InodeTable,
    InodeTableError,
    Size,
};
use nix::{
    libc::{
        EACCES,
        EBADFD,
        EINVAL,
        ENODATA,
        ENOENT,
        ENOSYS,
        ENOTDIR,
        O_ACCMODE,
        O_APPEND,
        O_RDWR,
        O_WRONLY,
    },
    unistd::{
        self,
        Gid,
        Uid,
    },
};
use si_frontend_types::{
    FuncKind,
    fs::{
        ActionKind,
        AttributeOutputTo,
        Binding,
        Bindings,
        FsApiError,
        Func,
        HydratedChangeSet,
        HydratedSchema,
        IdentityBindings,
        SchemaAttributes,
        kind_pluralized_to_string,
    },
};
pub use si_id::WorkspaceId;
use si_id::{
    ChangeSetId,
    FuncId,
    SchemaId,
};
use thiserror::Error;
use tokio::{
    runtime::{
        self,
    },
    sync::mpsc::UnboundedReceiver,
    task::JoinSet,
    time::Instant,
};

use crate::{
    async_wrapper::AsyncFuseWrapper,
    command::FilesystemCommand,
};

mod async_wrapper;
mod client;
mod command;
mod inode_table;

const FILE_HANDLE_READ_BIT: FileHandle = FileHandle::new(1 << 63);
const FILE_HANDLE_WRITE_BIT: FileHandle = FileHandle::new(1 << 62);

const FILE_STR_TS_INDEX: &str = "index.ts";
const FILE_STR_TS_INDEX_D_TS: &str = "index.d.ts";
const FILE_STR_DENO_CONFIG: &str = "deno.json";
const FILE_STR_ATTRS_JSON: &str = "attrs.json";
const FILE_STR_BINDINGS_JSON: &str = "bindings.json";
const FILE_STR_INSTALLED: &str = "INSTALLED";
const FILE_STR_PENDING_JSON: &str = "PENDING_BINDINGS_EDIT_ME.json";
const FILE_STR_SETTINGS_JSON: &str = "settings.json";

const TTL: Duration = Duration::from_secs(0);

const THIS_DIR: &str = ".";
const PARENT_DIR: &str = "..";

const DIR_STR_CHANGE_SETS: &str = "change-sets";
const DIR_STR_DEFINITION: &str = "definition";
const DIR_STR_FUNCTIONS: &str = "functions";
const DIR_STR_SCHEMAS: &str = "schemas";
const DIR_STR_LOCKED: &str = "locked";
const DIR_STR_UNLOCKED: &str = "unlocked";
const DIR_STR_VSCODE: &str = ".vscode";

const VSCODE_SETTINGS: &str = r#"{ "deno.enable": true }"#;

const DENO_CONFIG: &str = r#"{
    "compilerOptions": {
        "noImplicitAny": "false",
        "strict": "false",
        "supressImplicitAnyIndexErrors": "true",
        "lib": [
            "deno.window"
        ]
    }
}"#;
const DIR_STR_BLANK_CATEGORY: &str = "__UNCATEGORIZED__";

#[derive(Error, Debug)]
pub enum SiFileSystemError {
    #[error("failed to deserialize: {0}")]
    Deserialization(String),
    #[error("inode entry that should exist was not found: {0}")]
    ExpectedInodeNotFound(Inode),
    #[error("inode {0} is not a directory")]
    InodeNotDirectory(Inode),
    #[error("inode table error: {0}")]
    InodeTable(#[from] InodeTableError),
    #[error("osstring not utf8 compatible")]
    InvalidOsString,
    #[error("incorrect pending function kind")]
    PendingFuncKindWrong,
    #[error("failed to serialize: {0}")]
    Serialization(String),
    #[error("si-fs client error: {0}")]
    SiFsClient(#[from] SiFsClientError),
    #[error("std io error: {0}")]
    StdIo(#[from] std::io::Error),
    #[error("utf-8 error: {0}")]
    Utf8(#[from] Utf8Error),
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
    buf: Cursor<Vec<u8>>,
    append: bool,
    write: bool,
    dirty: bool,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct SiFileSystem {
    client: Arc<SiFsClient>,
    workspace_id: WorkspaceId,
    inode_table: Arc<InodeTable>,
    open_files: Arc<DashMap<FileHandle, OpenFile>>,
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
    fn new(
        mount_point: impl AsRef<Path>,
        token: String,
        endpoint: String,
        workspace_id: WorkspaceId,
        uid: Uid,
        gid: Gid,
    ) -> Self {
        let inode_table = InodeTable::new(
            mount_point,
            InodeEntryData::WorkspaceRoot { workspace_id },
            uid,
            gid,
        );

        let client = SiFsClient::new(token, workspace_id, endpoint).unwrap();

        Self {
            client: Arc::new(client),
            workspace_id,
            inode_table: Arc::new(inode_table),
            open_files: Arc::new(DashMap::new()),
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

    async fn hydrate_change_set(
        &self,
        change_sets_ino: Inode,
        change_set: HydratedChangeSet,
    ) -> SiFileSystemResult<()> {
        log::info!("Hydrating {} ({})", &change_set.name, change_set.id);
        let change_set_id = change_set.id;
        let change_set_ino = self.inode_table.upsert_with_parent_ino(
            change_sets_ino,
            &change_set.name,
            InodeEntryData::ChangeSet {
                change_set_id,
                name: change_set.name.clone(),
            },
            FileType::Directory,
            false,
            Size::Directory,
        )?;

        let functions_ino = self.inode_table.upsert_with_parent_ino(
            change_set_ino,
            DIR_STR_FUNCTIONS,
            InodeEntryData::ChangeSetFuncsDir { change_set_id },
            FileType::Directory,
            true,
            Size::Directory,
        )?;

        let mut join_set = JoinSet::new();

        for kind in [
            FuncKind::Action,
            FuncKind::Attribute,
            FuncKind::Authentication,
            FuncKind::CodeGeneration,
            FuncKind::Management,
            FuncKind::Qualification,
        ] {
            let kind_pluralize_str = kind_pluralized_to_string(kind);

            let kind_ino = self.inode_table.upsert_with_parent_ino(
                functions_ino,
                &kind_pluralize_str,
                InodeEntryData::ChangeSetFuncKindDir {
                    kind,
                    change_set_id,
                },
                FileType::Directory,
                true,
                Size::Directory,
            )?;

            for func in change_set.funcs.iter().filter(|f| f.kind == kind) {
                let self_clone = self.clone();
                let func = func.clone();
                join_set
                    .spawn(async move { self_clone.hydrate_func(kind_ino, change_set_id, &func) });
            }
        }

        let schemas_ino = self.inode_table.upsert_with_parent_ino(
            change_set_ino,
            DIR_STR_SCHEMAS,
            InodeEntryData::SchemasDir { change_set_id },
            FileType::Directory,
            true,
            Size::Directory,
        )?;

        let mut categorized: BTreeMap<String, Vec<HydratedSchema>> = BTreeMap::new();
        for schema in change_set.schemas {
            categorized
                .entry(schema.schema.category.clone())
                .and_modify(|schemas| schemas.push(schema.clone()))
                .or_insert(vec![schema]);
        }

        for (cat, schemas) in categorized {
            let self_clone = self.clone();
            join_set.spawn(async move {
                self_clone
                    .hydrate_schema_category(schemas_ino, change_set_id, cat, schemas)
                    .await
            });
        }

        join_set.join_all().await;

        Ok(())
    }

    async fn hydrate_schema_category(
        &self,
        schemas_ino: Inode,
        change_set_id: ChangeSetId,
        category: String,
        schemas: Vec<HydratedSchema>,
    ) -> SiFileSystemResult<()> {
        let cat_name = if category.trim().is_empty() {
            DIR_STR_BLANK_CATEGORY
        } else {
            &category
        };
        let cat_ino = self.upsert_category_dir(&category, cat_name, schemas_ino, change_set_id)?;

        let mut join_set = JoinSet::new();
        for schema in schemas {
            let self_clone = self.clone();
            join_set
                .spawn(async move { self_clone.hydrate_schema(cat_ino, change_set_id, schema) });
        }

        join_set.join_all().await;

        Ok(())
    }

    fn hydrate_schema(
        &self,
        category_ino: Inode,
        change_set_id: ChangeSetId,
        schema: HydratedSchema,
    ) -> SiFileSystemResult<()> {
        let schema_id = schema.schema.id;
        let schema_ino = self.inode_table.upsert_with_parent_ino(
            category_ino,
            &schema.schema.name,
            InodeEntryData::SchemaDir {
                schema_id,
                change_set_id,
                name: schema.schema.name.to_string(),
                installed: schema.schema.installed,
            },
            FileType::Directory,
            true,
            Size::Directory,
        )?;

        if schema.schema.installed {
            let functions_ino = self.inode_table.upsert_with_parent_ino(
                schema_ino,
                DIR_STR_FUNCTIONS,
                InodeEntryData::SchemaFuncsDir {
                    schema_id,
                    change_set_id,
                },
                FileType::Directory,
                true,
                Size::Directory,
            )?;

            for kind in [
                FuncKind::Action,
                FuncKind::Attribute,
                FuncKind::Authentication,
                FuncKind::CodeGeneration,
                FuncKind::Management,
                FuncKind::Qualification,
            ] {
                let kind_pluralize_str = kind_pluralized_to_string(kind);

                let kind_ino = self.inode_table.upsert_with_parent_ino(
                    functions_ino,
                    &kind_pluralize_str,
                    InodeEntryData::ChangeSetFuncKindDir {
                        kind,
                        change_set_id,
                    },
                    FileType::Directory,
                    true,
                    Size::Directory,
                )?;

                for func in schema
                    .funcs
                    .as_deref()
                    .unwrap_or(&[])
                    .iter()
                    .filter(|f| f.kind == kind)
                {
                    self.hydrate_func(kind_ino, change_set_id, func)?;
                }
            }

            let schema_def_ino = self.inode_table.upsert_with_parent_ino(
                schema_ino,
                DIR_STR_DEFINITION,
                InodeEntryData::SchemaDefinitionsDir {
                    schema_id,
                    change_set_id,
                },
                FileType::Directory,
                true,
                Size::Directory,
            )?;

            if let Some(unlocked_asset_func) = schema.asset_funcs.unlocked {
                self.inode_table.upsert_with_parent_ino(
                    schema_def_ino,
                    DIR_STR_UNLOCKED,
                    InodeEntryData::AssetDefinitionDir {
                        schema_id,
                        change_set_id,
                        size: unlocked_asset_func.code_size,
                        attrs_size: schema.asset_funcs.unlocked_attrs_size,
                        bindings_size: schema.asset_funcs.unlocked_bindings_size,
                        types_size: schema.asset_funcs.types_size,
                        unlocked: true,
                    },
                    FileType::Directory,
                    false,
                    Size::Directory,
                )?;
            }

            if let Some(locked_asset_func) = schema.asset_funcs.locked {
                self.inode_table.upsert_with_parent_ino(
                    schema_def_ino,
                    DIR_STR_LOCKED,
                    InodeEntryData::AssetDefinitionDir {
                        schema_id,
                        change_set_id,
                        size: locked_asset_func.code_size,
                        attrs_size: schema.asset_funcs.locked_attrs_size,
                        bindings_size: schema.asset_funcs.unlocked_bindings_size,
                        types_size: schema.asset_funcs.types_size,
                        unlocked: false,
                    },
                    FileType::Directory,
                    false,
                    Size::Directory,
                )?;
            };

            self.inode_table.upsert_with_parent_ino(
                schema_ino,
                FILE_STR_INSTALLED,
                InodeEntryData::InstalledSchemaMarker,
                FileType::RegularFile,
                false,
                Size::Force(0),
            )?;
        }

        Ok(())
    }

    fn hydrate_func(
        &self,
        func_kind_ino: Inode,
        change_set_id: ChangeSetId,
        func: &Func,
    ) -> SiFileSystemResult<()> {
        self.inode_table.upsert_with_parent_ino(
            func_kind_ino,
            &func.name,
            InodeEntryData::ChangeSetFuncDir {
                func_id: func.id,
                change_set_id,
                size: func.code_size,
                types_size: func.types_size,
                kind: func.kind,
            },
            FileType::Directory,
            true,
            Size::Directory,
        )?;
        Ok(())
    }

    async fn hydrate(&self) -> SiFileSystemResult<()> {
        log::info!(
            "Hydrating all funcs and schemas for all change sets... this will take some time"
        );

        let change_sets_ino = self.inode_table.upsert_with_parent_ino(
            Inode::new(1),
            DIR_STR_CHANGE_SETS,
            InodeEntryData::ChangeSets,
            FileType::Directory,
            true,
            Size::Directory,
        )?;

        let start = Instant::now();
        let hydration = self.client.hydrate_change_sets().await?;
        log::info!("Fetched hydration data in {:?}", start.elapsed());
        let mut join_set = JoinSet::new();
        for change_set in hydration {
            let self_clone = self.clone();
            join_set.spawn(async move {
                self_clone
                    .hydrate_change_set(change_sets_ino, change_set)
                    .await
            });
        }

        join_set.join_all().await;

        log::info!("Hydration took {:?}", start.elapsed());

        Ok(())
    }

    async fn getattr(
        &self,
        ino: Inode,
        _fh: Option<FileHandle>,
        reply: ReplyAttr,
    ) -> SiFileSystemResult<()> {
        let Some(entry) = self.inode_table.entry_for_ino(ino) else {
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
                if let Some(attrs) = self.inode_table.set_size(ino, size) {
                    reply.attr(&TTL, &attrs);
                } else {
                    reply.error(ENOENT);
                }
            }
        } else if let Some(entry) = self.inode_table.entry_for_ino(ino) {
            reply.attr(&TTL, entry.attrs());
        } else {
            reply.error(ENOENT);
        }

        Ok(())
    }

    async fn open(&self, ino: Inode, reply: ReplyOpen, flags: i32) -> SiFileSystemResult<()> {
        let Some(entry) = self.inode_table.entry_for_ino(ino) else {
            reply.error(ENOENT);
            return Ok(());
        };

        // Todo: handle append only
        let append = flags & O_APPEND != 0;
        // Are we opening with a file access mode including write access?
        let write_access = matches!(flags & O_ACCMODE, O_RDWR | O_WRONLY);
        // cannot detect O_TRUNC here. Instead we get SetAttr with size = 0;
        let mut fh = self.next_file_handle() | FILE_HANDLE_READ_BIT;
        if write_access {
            fh |= FILE_HANDLE_WRITE_BIT;
        }

        // Prefetch file data on open()
        let buf = Cursor::new(match entry.data() {
            InodeEntryData::WorkspaceDenoJson => {
                let root_path = self.inode_table.root_path();
                let func_entries: Vec<String> = self
                    .inode_table
                    .filter_entries(|_, entry| {
                        matches!(
                            entry.data(),
                            InodeEntryData::ChangeSetFuncDir { .. }
                                | InodeEntryData::SchemaFuncDir { .. }
                                | InodeEntryData::AssetDefinitionDir { .. }
                        )
                    })
                    .into_iter()
                    .filter_map(|(path, _)| {
                        path.strip_prefix(root_path).ok().and_then(|path| {
                            Path::new("./").join(path).to_str().map(|s| s.to_string())
                        })
                    })
                    .collect();

                let deno_json_value = serde_json::json!({ "workspace": func_entries });

                serde_json::to_string_pretty(&deno_json_value)
                    .map_err(|err| SiFileSystemError::Serialization(err.to_string()))?
                    .as_bytes()
                    .to_vec()
            }
            InodeEntryData::SchemaFuncBindingsPending { buf, .. } => buf.get_ref().clone(),
            InodeEntryData::InstalledSchemaMarker => "INSTALLED".as_bytes().to_vec(),
            InodeEntryData::AssetFuncCode {
                change_set_id,
                schema_id,
                unlocked,
            } => {
                let code = self
                    .client
                    .get_asset_func_code(*change_set_id, *schema_id, *unlocked)
                    .await?;
                code.as_bytes().to_vec()
            }
            InodeEntryData::AssetFuncTypes {
                change_set_id,
                schema_id,
                ..
            } => {
                let code = self
                    .client
                    .get_asset_func_types(*change_set_id, *schema_id)
                    .await?;
                code.as_bytes().to_vec()
            }
            InodeEntryData::FuncCode {
                change_set_id,
                func_id,
                ..
            } => {
                let code = self.client.get_func_code(*change_set_id, *func_id).await?;
                code.as_bytes().to_vec()
            }
            InodeEntryData::FuncTypes {
                change_set_id,
                func_id,
            } => self
                .client
                .get_func_types(*change_set_id, *func_id)
                .await?
                .as_bytes()
                .to_vec(),
            InodeEntryData::FuncTypesDenoConfig => DENO_CONFIG.as_bytes().to_vec(),
            InodeEntryData::VsCodeSettingsJson => VSCODE_SETTINGS.as_bytes().to_vec(),
            InodeEntryData::SchemaAttrsJson {
                schema_id,
                change_set_id,
                unlocked,
            } => {
                let attrs = self
                    .client
                    .get_schema_attrs(*change_set_id, *schema_id, *unlocked)
                    .await?;

                attrs
                    .to_vec_pretty()
                    .map_err(|err| SiFileSystemError::Serialization(err.to_string()))?
            }
            InodeEntryData::SchemaBindingsJson {
                schema_id,
                change_set_id,
                unlocked,
            } => {
                let bindings = self
                    .client
                    .get_identity_bindings(*change_set_id, *schema_id, *unlocked)
                    .await?;

                bindings
                    .to_vec_pretty()
                    .map_err(|err| SiFileSystemError::Serialization(err.to_string()))?
            }
            InodeEntryData::SchemaFuncBindings {
                change_set_id,
                func_id,
                schema_id,
                ..
            } => {
                let bindings = self
                    .client
                    .get_func_bindings(*change_set_id, *func_id, *schema_id)
                    .await?;

                bindings
                    .to_vec_pretty()
                    .map_err(|err| SiFileSystemError::Serialization(err.to_string()))?
            }
            // todo: prefetch directory listings?
            _ => vec![],
        });

        // Ensure the size is up to date for the next getattr call
        self.inode_table.set_size(ino, buf.get_ref().len() as u64);

        self.open_files.insert(
            fh,
            OpenFile {
                ino,
                fh,
                buf,
                append,
                write: write_access,
                dirty: false,
            },
        );

        reply.opened(fh.as_raw(), 0);

        Ok(())
    }

    async fn opendir(&self, _ino: Inode, reply: ReplyOpen, _flags: i32) -> SiFileSystemResult<()> {
        reply.opened((self.next_file_handle() | FILE_HANDLE_READ_BIT).as_raw(), 0);
        Ok(())
    }

    // Canonicalizes a relative path if that path is within the filesystem.
    // Returns None if the path is invalid or not within the scope of the fuse filesystem
    #[allow(unused)]
    async fn canonicalize(
        &self,
        cwd: Inode,
        target: PathBuf,
    ) -> SiFileSystemResult<Option<PathBuf>> {
        let mut cursor = Some(cwd);

        if target.is_absolute() {
            if self.inode_table.ino_for_path(target.as_path()).is_some() {
                return Ok(Some(target));
            } else {
                return Ok(None);
            }
        }

        for component in target.components() {
            cursor = match cursor {
                Some(cursor_ino) => match component {
                    std::path::Component::ParentDir => self.inode_table.parent_ino(cursor_ino),
                    std::path::Component::Normal(part) => {
                        self.upsert_dir_listing(cursor_ino).await?;
                        if let Some(cwd_path) = self.inode_table.path_buf_for_ino(cursor_ino) {
                            self.inode_table.ino_for_path(cwd_path.join(part))
                        } else {
                            None
                        }
                    }
                    _ => Some(cursor_ino),
                },
                None => return Ok(None),
            };
        }

        Ok(if let Some(cursor) = cursor {
            self.inode_table.path_for_ino(cursor)
        } else {
            None
        })
    }

    async fn create(
        &self,
        parent: Inode,
        name: OsString,
        _mode: u32,
        _umask: u32,
        _flags: i32,
        reply: ReplyCreate,
    ) -> SiFileSystemResult<()> {
        let mut parent_entry = {
            let Some(parent_entry) = self.inode_table.entry_for_ino(parent) else {
                reply.error(ENOENT);
                return Ok(());
            };

            parent_entry.to_owned()
        };

        let mut did_install = false;
        match parent_entry.data() {
            InodeEntryData::SchemaDir {
                schema_id,
                change_set_id,
                installed,
                ..
            } if name == FILE_STR_INSTALLED && !(*installed) => {
                self.client
                    .install_schema(*change_set_id, *schema_id)
                    .await?;

                did_install = true;

                let buf = Cursor::new("INSTALLED".as_bytes().to_vec());

                let ino = self.inode_table.upsert_with_parent_ino(
                    parent,
                    FILE_STR_INSTALLED,
                    InodeEntryData::InstalledSchemaMarker,
                    FileType::RegularFile,
                    false,
                    Size::Force(buf.get_ref().len() as u64),
                )?;
                let attrs = self.inode_table.make_attrs(
                    ino,
                    FileType::RegularFile,
                    false,
                    Size::Force(buf.get_ref().len() as u64),
                );

                let fh = self.next_file_handle() | FILE_HANDLE_READ_BIT;
                self.open_files.insert(
                    fh,
                    OpenFile {
                        ino,
                        fh,
                        buf,
                        append: false,
                        write: false,
                        dirty: false,
                    },
                );
                reply.created(&TTL, &attrs, 1, fh.as_raw(), 0);
            }
            _ => reply.error(EACCES),
        }

        if did_install {
            let new_data = match parent_entry.data.clone() {
                InodeEntryData::SchemaDir {
                    schema_id,
                    change_set_id,
                    name,
                    ..
                } => InodeEntryData::SchemaDir {
                    schema_id,
                    change_set_id,
                    name,
                    installed: true,
                },
                other => other,
            };
            parent_entry.data = new_data;

            self.inode_table.upsert_for_ino(parent, parent_entry)?;
        }

        Ok(())
    }

    async fn release(
        &self,
        ino: Inode,
        fh: FileHandle,
        _flags: i32,
        _lock_owner: Option<u64>,
        _flush: bool,
        reply: ReplyEmpty,
    ) -> SiFileSystemResult<()> {
        if let Some((_, open_file)) = self.open_files.remove(&fh) {
            let new_bytes = if open_file.write && open_file.dirty {
                let bytes = open_file.buf.get_ref().as_slice();

                let entry = self.inode_table.entry_for_ino(ino);
                match entry.as_ref().map(|entry| entry.data().clone()) {
                    Some(InodeEntryData::FuncCode {
                        change_set_id,
                        func_id,
                        ..
                    }) => {
                        self.client
                            .set_func_code(
                                change_set_id,
                                func_id,
                                std::str::from_utf8(bytes)?.to_string(),
                            )
                            .await?;
                        Some(bytes.len())
                    }
                    Some(InodeEntryData::AssetFuncCode {
                        change_set_id,
                        schema_id,
                        unlocked,
                    }) if unlocked => {
                        self.client
                            .set_asset_func_code(
                                change_set_id,
                                schema_id,
                                std::str::from_utf8(bytes)?.to_string(),
                            )
                            .await?;
                        Some(bytes.len())
                    }
                    Some(InodeEntryData::SchemaAttrsJson {
                        schema_id,
                        change_set_id,
                        unlocked,
                    }) if unlocked => {
                        let attrs = SchemaAttributes::from_bytes(bytes)
                            .map_err(|err| SiFileSystemError::Deserialization(err.to_string()))?;

                        self.client
                            .set_schema_attrs(change_set_id, schema_id, attrs)
                            .await?;

                        Some(bytes.len())
                    }
                    Some(InodeEntryData::SchemaBindingsJson {
                        schema_id,
                        change_set_id,
                        unlocked,
                    }) if unlocked => {
                        let bindings = IdentityBindings::from_bytes(bytes)
                            .map_err(|err| SiFileSystemError::Deserialization(err.to_string()))?;

                        self.client
                            .set_identity_bindings(change_set_id, schema_id, bindings)
                            .await?;

                        Some(bytes.len())
                    }
                    Some(InodeEntryData::SchemaFuncBindings {
                        change_set_id,
                        func_id,
                        kind,
                        schema_id,
                        unlocked,
                        ..
                    }) if unlocked => {
                        let bindings = Bindings::from_bytes(bytes)
                            .map_err(|err| SiFileSystemError::Deserialization(err.to_string()))?;

                        if bindings.kind_matches(kind) {
                            self.client
                                .set_func_bindings(
                                    change_set_id,
                                    func_id,
                                    schema_id,
                                    bindings,
                                    false,
                                )
                                .await?;
                        }

                        Some(bytes.len())
                    }
                    Some(InodeEntryData::SchemaFuncBindingsPending {
                        change_set_id,
                        schema_id,
                        kind,
                        pending_func_id,
                        ..
                    }) => {
                        if self
                            .create_pending_func_on_release(
                                ino,
                                bytes,
                                change_set_id,
                                schema_id,
                                kind,
                                pending_func_id,
                            )
                            .await?
                        {
                            None
                        } else {
                            Some(bytes.len())
                        }
                    }
                    _ => None,
                }
            } else {
                None
            };

            if let Some(new_bytes) = new_bytes {
                self.inode_table.set_size(ino, new_bytes as u64);
            }
        }

        reply.ok();

        Ok(())
    }

    async fn create_pending_func_on_release(
        &self,
        ino: Inode,
        open_file_bytes: &[u8],
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
        kind: FuncKind,
        pending_func_id: Option<FuncId>,
    ) -> SiFileSystemResult<bool> {
        #[derive(Debug)]
        enum WriteOutcome {
            SerializationError(String),
            BindingsEmpty,
            CreateFuncFailed(SiFileSystemError),
            ParentInoNotFound,
            Success,
        }

        let outcome = match Bindings::from_bytes(open_file_bytes) {
            Ok(bindings) => {
                if !bindings.bindings.is_empty() {
                    let parent_ino = self.inode_table.parent_ino(ino);

                    if let Some(parent_ino) = parent_ino {
                        let func_name = self
                            .inode_table
                            .entry_for_ino(parent_ino)
                            .map(|entry| entry.name.to_owned())
                            .unwrap_or("unknown".into());

                        match self
                            .create_func_or_pending_attributes(
                                parent_ino,
                                kind,
                                change_set_id,
                                schema_id,
                                Some(bindings.bindings[0].clone()),
                                func_name,
                                pending_func_id,
                            )
                            .await
                        {
                            Ok(_) => {
                                self.inode_table.invalidate_ino(ino);
                                WriteOutcome::Success
                            }
                            Err(err) => WriteOutcome::CreateFuncFailed(err),
                        }
                    } else {
                        WriteOutcome::ParentInoNotFound
                    }
                } else {
                    WriteOutcome::BindingsEmpty
                }
            }
            Err(err) => WriteOutcome::SerializationError(err.to_string()),
        };

        if let WriteOutcome::Success = outcome {
            Ok(true)
        } else {
            self.inode_table.modify_ino(ino, |entry| {
                if let InodeEntryData::SchemaFuncBindingsPending { buf, .. } = &mut entry.data {
                    *buf = Arc::new(Cursor::new(open_file_bytes.to_vec()))
                }
            });

            match outcome {
                WriteOutcome::SerializationError(err) => Err(SiFileSystemError::Serialization(err)),
                WriteOutcome::BindingsEmpty => {
                    log::debug!("bindings were empty, cannot create func without bindings");
                    Ok(false)
                }
                WriteOutcome::CreateFuncFailed(si_file_system_error) => Err(si_file_system_error),
                WriteOutcome::ParentInoNotFound => {
                    log::trace!("could not find parent for func. This shouldn't happen");
                    Ok(false)
                }
                WriteOutcome::Success => Ok(false),
            }
        }
    }

    async fn mkdir(
        &self,
        parent: Inode,
        name: OsString,
        _mode: u32,
        _umask: u32,
        reply: ReplyEntry,
    ) -> SiFileSystemResult<()> {
        let name = name
            .into_string()
            .map_err(|_| SiFileSystemError::InvalidOsString)?;

        let parent_entry = {
            let Some(parent_entry) = self.inode_table.entry_for_ino(parent) else {
                reply.error(ENOENT);
                return Ok(());
            };

            parent_entry.to_owned()
        };

        match parent_entry.data() {
            InodeEntryData::ChangeSets => {
                let attrs = self.create_change_set(name, parent).await?;

                reply.entry(&TTL, &attrs, 1);
            }
            InodeEntryData::SchemaCategoryDir {
                change_set_id,
                category,
                ..
            } => {
                let attrs = self
                    .create_schema(change_set_id, name, parent, category)
                    .await?;

                reply.entry(&TTL, &attrs, 1);
            }
            InodeEntryData::SchemasDir { change_set_id } => {
                let ino = self.inode_table.upsert_with_parent_ino(
                    parent,
                    name.clone(),
                    InodeEntryData::SchemaCategoryDir {
                        change_set_id: *change_set_id,
                        category: name,
                        pending: true,
                    },
                    FileType::Directory,
                    true,
                    Size::Directory,
                )?;

                reply.entry(
                    &TTL,
                    &self
                        .inode_table
                        .make_attrs(ino, FileType::Directory, true, Size::Directory),
                    1,
                )
            }
            InodeEntryData::SchemaDefinitionsDir {
                schema_id,
                change_set_id,
            } => {
                if name == DIR_STR_UNLOCKED {
                    let asset_funcs = self
                        .client
                        .unlock_schema(*change_set_id, *schema_id)
                        .await?;
                    if let Some(unlocked_asset_func) = asset_funcs.unlocked {
                        let ino = self.inode_table.upsert_with_parent_ino(
                            parent,
                            DIR_STR_UNLOCKED,
                            InodeEntryData::AssetDefinitionDir {
                                schema_id: *schema_id,
                                change_set_id: *change_set_id,
                                size: unlocked_asset_func.code_size,
                                attrs_size: asset_funcs.unlocked_attrs_size,
                                bindings_size: asset_funcs.unlocked_bindings_size,
                                types_size: asset_funcs.types_size,
                                unlocked: true,
                            },
                            FileType::Directory,
                            false,
                            Size::Directory,
                        )?;

                        let attrs = self.inode_table.make_attrs(
                            ino,
                            FileType::Directory,
                            true,
                            Size::Directory,
                        );

                        reply.entry(&TTL, &attrs, 1);
                    } else {
                        reply.error(EINVAL);
                    }
                } else {
                    reply.error(EACCES);
                }
            }
            InodeEntryData::SchemaFuncKindDir {
                kind,
                schema_id,
                change_set_id,
            } => {
                let binding = match kind {
                    FuncKind::Action => None,
                    FuncKind::Attribute => None,
                    FuncKind::Authentication => Some(Binding::Authentication),
                    FuncKind::CodeGeneration => Some(Binding::CodeGeneration { inputs: vec![] }),
                    FuncKind::Qualification => Some(Binding::Qualification { inputs: vec![] }),
                    FuncKind::Management => Some(Binding::Management),
                    _ => {
                        reply.error(EINVAL);
                        return Ok(());
                    }
                };

                let attrs = self
                    .create_func_or_pending_attributes(
                        parent,
                        *kind,
                        *change_set_id,
                        *schema_id,
                        binding,
                        name,
                        None,
                    )
                    .await?;

                reply.entry(&TTL, &attrs, 1);
            }
            InodeEntryData::SchemaFuncVariantsDir {
                locked_id,
                change_set_id,
                schema_id,
                kind,
                ..
            } => {
                if name == DIR_STR_UNLOCKED {
                    if let Some(locked_id) = locked_id {
                        let unlocked_func = self
                            .client
                            .unlock_func(*change_set_id, *schema_id, *locked_id)
                            .await?;

                        let ino = self.inode_table.upsert_with_parent_ino(
                            parent,
                            DIR_STR_UNLOCKED,
                            InodeEntryData::SchemaFuncDir {
                                kind: *kind,
                                change_set_id: *change_set_id,
                                func_id: unlocked_func.id,
                                size: unlocked_func.code_size,
                                types_size: unlocked_func.types_size,
                                unlocked: true,
                                schema_id: *schema_id,
                                bindings_size: 0,
                            },
                            FileType::Directory,
                            false,
                            Size::Directory,
                        )?;

                        let attrs = self.inode_table.make_attrs(
                            ino,
                            FileType::Directory,
                            true,
                            Size::Directory,
                        );

                        reply.entry(&TTL, &attrs, 1);
                    } else {
                        reply.error(EINVAL);
                    }
                } else {
                    reply.error(EACCES);
                }
            }
            InodeEntryData::WorkspaceRoot { .. }
            | InodeEntryData::WorkspaceDenoJson
            | InodeEntryData::VsCodeDir
            | InodeEntryData::ChangeSet { .. }
            | InodeEntryData::SchemaFuncBindingsPending { .. }
            | InodeEntryData::AssetFuncCode { .. }
            | InodeEntryData::SchemaFuncDir { .. }
            | InodeEntryData::AssetDefinitionDir { .. }
            | InodeEntryData::SchemaFuncsDir { .. }
            | InodeEntryData::SchemaDir { .. }
            | InodeEntryData::ChangeSetFuncDir { .. }
            | InodeEntryData::ChangeSetFuncsDir { .. }
            | InodeEntryData::ChangeSetFuncKindDir { .. }
            | InodeEntryData::FuncCode { .. }
            | InodeEntryData::AssetFuncTypes { .. } => reply.error(EACCES),

            InodeEntryData::SchemaAttrsJson { .. }
            | InodeEntryData::VsCodeSettingsJson
            | InodeEntryData::FuncTypes { .. }
            | InodeEntryData::FuncTypesDenoConfig
            | InodeEntryData::SchemaBindingsJson { .. }
            | InodeEntryData::InstalledSchemaMarker
            | InodeEntryData::SchemaFuncBindings { .. } => {
                reply.error(ENOTDIR);
            }
        }

        Ok(())
    }

    async fn create_schema(
        &self,
        change_set_id: &ChangeSetId,
        name: String,
        parent: Inode,
        category: &str,
    ) -> SiFileSystemResult<FileAttr> {
        let created_schema = self
            .client
            .create_schema(*change_set_id, name, category)
            .await?;

        let ino = self.inode_table.upsert_with_parent_ino(
            parent,
            &created_schema.name,
            InodeEntryData::SchemaDir {
                schema_id: created_schema.schema_id,
                change_set_id: *change_set_id,
                name: created_schema.name.to_string(),
                installed: true,
            },
            FileType::Directory,
            true,
            Size::Directory,
        )?;

        if let Some(mut cat_dir) = self.inode_table.entry_for_ino(parent) {
            if let InodeEntryData::SchemaCategoryDir { pending: true, .. } = cat_dir.data() {
                cat_dir.data = InodeEntryData::SchemaCategoryDir {
                    change_set_id: *change_set_id,
                    category: category.to_string(),
                    pending: false,
                };

                self.inode_table.upsert_for_ino(ino, cat_dir)?;
            }
        }

        Ok(self
            .inode_table
            .make_attrs(ino, FileType::Directory, true, Size::Directory))
    }

    async fn create_change_set(&self, name: String, parent: Inode) -> SiFileSystemResult<FileAttr> {
        let change_set = self.client.create_change_set(name.to_owned()).await?;
        let attrs = {
            let ino = self.inode_table.upsert_with_parent_ino(
                parent,
                &name,
                InodeEntryData::ChangeSet {
                    change_set_id: change_set.id,
                    name: name.to_owned(),
                },
                FileType::Directory,
                true,
                Size::Directory,
            )?;
            self.inode_table
                .make_attrs(ino, FileType::Directory, true, Size::Directory)
        };
        Ok(attrs)
    }

    #[allow(clippy::too_many_arguments)]
    async fn create_func_or_pending_attributes(
        &self,
        parent: Inode,
        kind: FuncKind,
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
        binding: Option<Binding>,
        name: String,
        pending_func_id: Option<FuncId>,
    ) -> SiFileSystemResult<FileAttr> {
        let maybe_created_func = match (binding, pending_func_id) {
            (Some(binding), None) => Some(
                self.client
                    .create_func(change_set_id, schema_id, kind, name.clone(), binding)
                    .await?,
            ),
            (Some(binding), Some(pending_func_id)) => {
                self.client
                    .set_func_bindings(
                        change_set_id,
                        pending_func_id,
                        schema_id,
                        Bindings {
                            bindings: vec![binding],
                        },
                        true,
                    )
                    .await?
            }
            _ => None,
        };

        let ino = self.inode_table.upsert_with_parent_ino(
            parent,
            &name,
            InodeEntryData::SchemaFuncVariantsDir {
                kind,
                locked_id: None,
                unlocked_id: None,
                locked_size: 0,
                unlocked_size: maybe_created_func
                    .as_ref()
                    .map(|f| f.code_size)
                    .unwrap_or(0),
                schema_id,
                locked_bindings_size: 0,
                locked_types_size: 0,
                unlocked_types_size: maybe_created_func
                    .as_ref()
                    .map(|f| f.types_size)
                    .unwrap_or(0),
                unlocked_bindings_size: maybe_created_func
                    .as_ref()
                    .map(|f| f.bindings_size)
                    .unwrap_or(0),
                change_set_id,
                pending: maybe_created_func.is_none(),
                pending_func_id: if maybe_created_func.is_none() {
                    pending_func_id
                } else {
                    None
                },
            },
            FileType::Directory,
            true,
            Size::Directory,
        )?;

        Ok(self
            .inode_table
            .make_attrs(ino, FileType::Directory, true, Size::Directory))
    }

    async fn lookup(
        &self,
        parent: Inode,
        name: impl AsRef<Path>,
        reply: ReplyEntry,
    ) -> SiFileSystemResult<()> {
        let Some(parent_path) = self.inode_table.path_buf_for_ino(parent) else {
            reply.error(ENOENT);
            return Ok(());
        };

        let name = name.as_ref();
        let full_path = parent_path.join(name);
        let maybe_ino = self.inode_table.ino_for_path(&full_path);
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

        if let Some(entry) = self.inode_table.entry_for_ino(entry_ino) {
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
        let Some(entry) = self.inode_table.entry_for_ino(ino) else {
            reply.error(ENOENT);
            return Ok(());
        };

        if entry.data().openable() {
            match self.open_files.get(&fh) {
                // File handle contents is being tracked
                Some(of) => {
                    reply.data(get_read_slice(
                        of.value().buf.get_ref(),
                        offset as usize,
                        size as usize,
                    ));
                }
                // File was somehow not opened yet?
                None => {
                    reply.error(ENODATA);
                }
            }
        } else {
            reply.error(EINVAL);
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn write(
        &self,
        ino: Inode,
        fh: FileHandle,
        offset: i64,
        data: Vec<u8>,
        _write_flags: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyWrite,
    ) -> SiFileSystemResult<()> {
        if offset < 0 {
            reply.error(EINVAL);
            return Ok(());
        }

        let written = {
            let Some(entry) = self.inode_table.entry_for_ino(ino) else {
                reply.error(ENOENT);
                return Ok(());
            };

            let Some(mut open_file) = self.open_files.get_mut(&fh) else {
                reply.error(EBADFD);
                return Ok(());
            };

            if entry.attrs().size == 0 {
                open_file.buf.get_mut().truncate(0);
            }

            open_file.dirty = true;

            open_file
                .buf
                .seek(std::io::SeekFrom::Start(offset as u64))?;

            let written = open_file.buf.write(data.as_slice())? as u32;

            self.inode_table.set_size(ino, written as u64);

            written
        };

        reply.written(written);

        Ok(())
    }

    // currently only exists to support attaching existing functions
    async fn rename(
        &self,
        parent: Inode,
        name: OsString,
        newparent: Inode,
        _newname: OsString,
        _flags: u32,
        reply: ReplyEmpty,
    ) -> SiFileSystemResult<()> {
        let source_path = self
            .inode_table
            .path_for_ino(parent)
            .ok_or(SiFileSystemError::ExpectedInodeNotFound(parent))?
            .join(name);

        let Some(source_ino) = self.inode_table.ino_for_path(&source_path) else {
            reply.error(ENOENT);
            return Ok(());
        };

        let Some(source_entry) = self.inode_table.entry_for_ino(source_ino) else {
            reply.error(ENOENT);
            return Ok(());
        };

        let Some(dest_parent_entry) = self.inode_table.entry_for_ino(newparent) else {
            reply.error(ENOENT);
            return Ok(());
        };

        let (source_func_id, source_func_kind, source_change_set_id) = match source_entry.data() {
            InodeEntryData::ChangeSetFuncDir {
                func_id,
                change_set_id,
                kind,
                ..
            } => (*func_id, *kind, *change_set_id),
            InodeEntryData::FuncCode {
                change_set_id,
                func_id,
                kind,
                ..
            } => (*func_id, *kind, *change_set_id),
            InodeEntryData::SchemaFuncDir {
                change_set_id,
                func_id,
                kind,
                ..
            } => (*func_id, *kind, *change_set_id),
            _ => {
                reply.error(EINVAL);
                return Ok(());
            }
        };

        let (dest_schema_id, dest_change_set_id) = match dest_parent_entry.data() {
            InodeEntryData::SchemaFuncKindDir {
                kind,
                schema_id,
                change_set_id,
            } => {
                if *kind != source_func_kind {
                    reply.error(EINVAL);
                    return Ok(());
                }
                (*schema_id, *change_set_id)
            }
            _ => {
                reply.error(EINVAL);
                return Ok(());
            }
        };

        if dest_change_set_id != source_change_set_id {
            reply.error(EINVAL);
            return Ok(());
        }

        let binding = match source_func_kind {
            FuncKind::Action => None,
            FuncKind::Attribute => None,
            FuncKind::Authentication => Some(Binding::Authentication),
            FuncKind::CodeGeneration => Some(Binding::CodeGeneration { inputs: vec![] }),
            FuncKind::Qualification => Some(Binding::Qualification { inputs: vec![] }),
            FuncKind::Management => Some(Binding::Management),
            _ => {
                reply.error(EINVAL);
                return Ok(());
            }
        };

        match binding {
            Some(binding) => {
                self.client
                    .set_func_bindings(
                        dest_change_set_id,
                        source_func_id,
                        dest_schema_id,
                        Bindings {
                            bindings: vec![binding],
                        },
                        true,
                    )
                    .await?;
            }
            None => {
                // Create a pending attach
                let name = self
                    .inode_table
                    .entries_by_path()
                    .iter()
                    .find(|entry| match entry.data() {
                        InodeEntryData::ChangeSetFuncDir {
                            func_id: entry_func_id,
                            ..
                        } => source_func_id == *entry_func_id,
                        InodeEntryData::SchemaFuncVariantsDir {
                            locked_id,
                            unlocked_id,
                            ..
                        } => {
                            Some(source_func_id) == *locked_id
                                || Some(source_func_id) == *unlocked_id
                        }
                        _ => false,
                    })
                    .map(|entry| entry.name.to_owned())
                    .unwrap_or("unknown".into());

                self.create_func_or_pending_attributes(
                    newparent,
                    source_func_kind,
                    dest_change_set_id,
                    dest_schema_id,
                    None,
                    name,
                    Some(source_func_id),
                )
                .await?;
            }
        }

        reply.ok();

        Ok(())
    }

    async fn upsert_dir_listing(&self, ino: Inode) -> SiFileSystemResult<DirListing> {
        let entry = self
            .inode_table
            .entry_for_ino(ino)
            .ok_or(SiFileSystemError::ExpectedInodeNotFound(ino))?;

        // if entry.updated_at().elapsed() < Duration::from_secs(8) {
        //     for entry in self.inode_table.direct_child_entries(ino)? {
        //         dirs.add(entry.ino, entry.name, entry.kind);
        //     }
        //     return Ok(dirs);
        // }

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
                self.upsert_workspace_root(&entry, &mut dirs).await?;
            }
            // `/change-sets/`
            InodeEntryData::ChangeSets => {
                self.upsert_change_sets_dir(ino, &mut dirs).await?;
            }
            // `/change-sets/$change_set_name/`
            InodeEntryData::ChangeSet { change_set_id, .. } => {
                self.upsert_change_set_dir(&entry, change_set_id, &mut dirs).await?;
            }
            // `/change-sets/$change_set_name/functions/`
            InodeEntryData::ChangeSetFuncsDir { change_set_id } => {
                self.upsert_change_set_funcs_dir(&entry, change_set_id, &mut dirs).await?;
            }
            // `/change-sets/$change_set_name/functions/$func_kind/`
            InodeEntryData::ChangeSetFuncKindDir {
                kind,
                change_set_id,
            } => {
                self.upsert_change_set_func_kind_dir(change_set_id, kind, &entry, &mut dirs).await?;
            }
            // `/change-sets/$change_set_name/functions/$func_kind/$func_name/`
            InodeEntryData::ChangeSetFuncDir {
                func_id: id,
                change_set_id,
                size,
                types_size,
                kind,
            } => {
                self.upsert_change_set_func_dir(&entry, *id, *change_set_id, *size, &mut dirs, *kind, *types_size).await?;
            }
            // `/change-sets/$change_set_name/schemas/`
            InodeEntryData::SchemasDir { change_set_id } => {
                self.upsert_schemas_dir(change_set_id, &entry, &mut dirs).await?;
            }
            // `/change-sets/$change_set_name/schemas/$schema_name/`
            InodeEntryData::SchemaDir {
                schema_id, change_set_id, installed, ..
            } => {
                self.upsert_schema_dir(installed, &entry, schema_id, change_set_id, &mut dirs).await?;
            }
            // `/change-sets/$change_set_name/schemas/$schema_name/definition/`
            InodeEntryData::SchemaDefinitionsDir { schema_id, change_set_id } => {
                self.upsert_schema_def_dir(change_set_id, schema_id, &entry, &mut dirs).await?;
            }
            // `/change-sets/$change_set_name/schemas/$schema_name/functions/`
            InodeEntryData::SchemaFuncsDir {
                schema_id,
                change_set_id,
                ..
            } => {
                self.upsert_schema_funcs_dir(&entry, schema_id, change_set_id, &mut dirs).await?;
            }
            // `/change-sets/$change_set_name/schemas/$schema_name/functions/$func_kind/`
            InodeEntryData::SchemaFuncKindDir {
                kind,
                schema_id,
                change_set_id,
            } => {
                self.upsert_schema_func_kind_dir(change_set_id, schema_id, kind, &entry, &mut dirs).await?;
            }
            // `/change-sets/$change_set_name/schemas/$schema_name/functions/$func_kind/$func_name`
            InodeEntryData::SchemaFuncVariantsDir {
                locked_id,
                unlocked_id,
                schema_id,
                change_set_id,
                locked_size,
                locked_bindings_size,
                locked_types_size,
                unlocked_size,
                unlocked_bindings_size,
                unlocked_types_size,
                kind,
                pending,
                pending_func_id,
            } => {
                self.upsert_schema_func_variants_dir(
                    *pending,
                    *kind,
                    ino,
                    &entry,
                    *change_set_id,
                    *schema_id,
                    &mut dirs,
                    *locked_id,
                    *locked_size,
                    *locked_bindings_size,
                    *locked_types_size,
                    *unlocked_id,
                    *unlocked_size,
                    *unlocked_bindings_size,
                    *unlocked_types_size,
                    pending_func_id.to_owned(),
                ).await?;
            }
            // `/change-sets/$change_set_name/schemas/$schema_name/functions/$func_kind/$func_name/{locked
            // | unlocked}/`
            InodeEntryData::SchemaFuncDir {
                kind,
                change_set_id,
                func_id,
                schema_id,
                size,
                types_size,
                bindings_size,
                unlocked
             } => {
                self.upsert_schema_func_dir(
                    &entry,
                    *func_id,
                    *change_set_id,
                    *unlocked,
                    *size,
                    &mut dirs,
                    *kind,
                    *schema_id,
                    *bindings_size,
                    *types_size,
                ).await?;
            }
            // `/change-sets/$change_set_name/schemas/$schema_name/definition//{locked | unlocked}/`
            InodeEntryData::AssetDefinitionDir {
                change_set_id,
                schema_id,
                unlocked,
                size,
                attrs_size,
                bindings_size,
                types_size,
            } => {
                self.upsert_asset_def_dir(
                    &entry,
                    *change_set_id,
                    *schema_id,
                    *unlocked,
                    *size,
                    &mut dirs,
                    *attrs_size,
                    *bindings_size,
                    *types_size,
                ).await?;
            }
            InodeEntryData::SchemaCategoryDir { category, change_set_id, .. } => {
                self.upsert_schema_category_dir(
                    entry.ino,
                    *change_set_id,
                    category,
                    &mut dirs
                ).await?
            }
            InodeEntryData::VsCodeDir => {
                self.upsert_vscode_dir(entry.ino, &mut dirs)?;
            }
            // `/change-sets/$change_set_name/schemas/$schema_name/INSTALLED`
            InodeEntryData::InstalledSchemaMarker
            // `/.vscode/settings.json`
            | InodeEntryData::VsCodeSettingsJson
            // `/change-sets/$change_set_name/functions/$func_kind/$func_name/{locked|unlocked}/index.ts`
            | InodeEntryData::FuncCode { .. }
            // `/change-sets/$change_set_name/functions/$func_kind/$func_name/{locked|unlocked}/index.d.ts`
            | InodeEntryData::FuncTypes { .. }
            // `/change-sets/$change_set_name/functions/$func_kind/$func_name/{locked|unlocked}/deno.json`
            | InodeEntryData::FuncTypesDenoConfig
            // `/change-sets/$change_set_name/functions/$func_kind/$func_name/(locked|unlocked}/attrs.json`
            | InodeEntryData::SchemaFuncBindings { .. }
            // `/change-sets/$change_set_name/schemas/$schema_name/$func_name/PENDING_BINDINGS_EDIT_ME.json`
            | InodeEntryData::SchemaFuncBindingsPending { .. }
            // `/change-sets/$change_set_name/schemas/$schema_name/definition/{locked|unlocked}/attrs.json`
            | InodeEntryData::SchemaAttrsJson { .. }
            // `/change-sets/$change_set_name/schemas/$schema_name/definition/{locked|unlocked}/bindings.json`
            | InodeEntryData::SchemaBindingsJson { .. }
            // `/change-sets/$change_set_name/schemas/$schema_name/definition/{locked|unlocked}/index.ts`
            | InodeEntryData::AssetFuncCode { .. }
            // `/change-sets/$change_set_name/schemas/$schema_name/definition/{locked|unlocked}/index.d.ts`
            | InodeEntryData::AssetFuncTypes { .. }
            | InodeEntryData::WorkspaceDenoJson => {
                // a file is not a directory!
                return Err(SiFileSystemError::InodeNotDirectory(ino));
            }

        }

        Ok(dirs)
    }

    fn upsert_vscode_dir(&self, entry_ino: Inode, dirs: &mut DirListing) -> SiFileSystemResult<()> {
        let ino = self.inode_table.upsert_with_parent_ino(
            entry_ino,
            FILE_STR_SETTINGS_JSON,
            InodeEntryData::VsCodeSettingsJson,
            FileType::RegularFile,
            false,
            Size::UseExisting(VSCODE_SETTINGS.len() as u64),
        )?;

        dirs.add(ino, FILE_STR_SETTINGS_JSON.into(), FileType::RegularFile);

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn upsert_asset_def_dir(
        &self,
        entry: &InodeEntry,
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
        unlocked: bool,
        size: u64,
        dirs: &mut DirListing,
        attrs_size: u64,
        bindings_size: u64,
        types_size: u64,
    ) -> SiFileSystemResult<()> {
        let inode_table = &self.inode_table;

        let ino = inode_table.upsert_with_parent_ino(
            entry.ino,
            FILE_STR_TS_INDEX,
            InodeEntryData::AssetFuncCode {
                change_set_id,
                schema_id,
                unlocked,
            },
            FileType::RegularFile,
            unlocked,
            Size::UseExisting(size),
        )?;
        dirs.add(ino, FILE_STR_TS_INDEX.into(), FileType::RegularFile);

        let ino = inode_table.upsert_with_parent_ino(
            entry.ino,
            FILE_STR_TS_INDEX_D_TS,
            InodeEntryData::AssetFuncTypes {
                change_set_id,
                schema_id,
                unlocked,
            },
            FileType::RegularFile,
            false,
            Size::UseExisting(types_size),
        )?;
        dirs.add(ino, FILE_STR_TS_INDEX_D_TS.into(), FileType::RegularFile);

        let ino = inode_table.upsert_with_parent_ino(
            entry.ino,
            FILE_STR_DENO_CONFIG,
            InodeEntryData::FuncTypesDenoConfig,
            FileType::RegularFile,
            false,
            Size::UseExisting(DENO_CONFIG.len() as u64),
        )?;
        dirs.add(ino, FILE_STR_DENO_CONFIG.into(), FileType::RegularFile);

        let ino = inode_table.upsert_with_parent_ino(
            entry.ino,
            FILE_STR_ATTRS_JSON,
            InodeEntryData::SchemaAttrsJson {
                schema_id,
                change_set_id,
                unlocked,
            },
            FileType::RegularFile,
            unlocked,
            Size::UseExisting(attrs_size),
        )?;
        dirs.add(ino, FILE_STR_ATTRS_JSON.into(), FileType::RegularFile);

        let ino = inode_table.upsert_with_parent_ino(
            entry.ino,
            FILE_STR_BINDINGS_JSON,
            InodeEntryData::SchemaBindingsJson {
                schema_id,
                change_set_id,
                unlocked,
            },
            FileType::RegularFile,
            unlocked,
            Size::UseExisting(bindings_size),
        )?;

        dirs.add(ino, FILE_STR_BINDINGS_JSON.into(), FileType::RegularFile);

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn upsert_schema_func_dir(
        &self,
        entry: &InodeEntry,
        func_id: si_id::FuncId,
        change_set_id: ChangeSetId,
        unlocked: bool,
        size: u64,
        dirs: &mut DirListing,
        kind: FuncKind,
        schema_id: SchemaId,
        bindings_size: u64,
        types_size: u64,
    ) -> SiFileSystemResult<()> {
        let inode_table = &self.inode_table;
        let ino = inode_table.upsert_with_parent_ino(
            entry.ino,
            FILE_STR_TS_INDEX,
            InodeEntryData::FuncCode {
                func_id,
                change_set_id,
                kind,
            },
            FileType::RegularFile,
            unlocked,
            Size::UseExisting(size),
        )?;
        dirs.add(ino, FILE_STR_TS_INDEX.into(), FileType::RegularFile);

        let ino = inode_table.upsert_with_parent_ino(
            entry.ino,
            FILE_STR_TS_INDEX_D_TS,
            InodeEntryData::FuncTypes {
                func_id,
                change_set_id,
            },
            FileType::RegularFile,
            false,
            Size::UseExisting(types_size),
        )?;
        dirs.add(ino, FILE_STR_TS_INDEX_D_TS.into(), FileType::RegularFile);

        let ino = inode_table.upsert_with_parent_ino(
            entry.ino,
            FILE_STR_DENO_CONFIG,
            InodeEntryData::FuncTypesDenoConfig,
            FileType::RegularFile,
            false,
            Size::UseExisting(DENO_CONFIG.len() as u64),
        )?;
        dirs.add(ino, FILE_STR_DENO_CONFIG.into(), FileType::RegularFile);

        let ino = inode_table.upsert_with_parent_ino(
            entry.ino,
            FILE_STR_BINDINGS_JSON,
            InodeEntryData::SchemaFuncBindings {
                kind,
                change_set_id,
                func_id,
                schema_id,
                size: bindings_size,
                unlocked,
            },
            FileType::RegularFile,
            unlocked,
            Size::UseExisting(bindings_size),
        )?;
        dirs.add(ino, FILE_STR_BINDINGS_JSON.into(), FileType::RegularFile);

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn upsert_schema_func_variants_dir(
        &self,
        pending: bool,
        kind: FuncKind,
        ino: Inode,
        entry: &InodeEntry,
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
        dirs: &mut DirListing,
        locked_id: Option<si_id::FuncId>,
        locked_size: u64,
        locked_binding_size: u64,
        locked_types_size: u64,
        unlocked_id: Option<si_id::FuncId>,
        unlocked_size: u64,
        unlocked_bindings_size: u64,
        unlocked_types_size: u64,
        pending_func_id: Option<FuncId>,
    ) -> SiFileSystemResult<()> {
        let inode_table = &self.inode_table;
        if pending {
            let default_bindings = match kind {
                FuncKind::Action => Bindings {
                    bindings: vec![default_action_bindings()],
                },
                FuncKind::Attribute => Bindings {
                    bindings: vec![default_attribute_bindings()],
                },
                _ => return Err(SiFileSystemError::PendingFuncKindWrong),
            };

            let buf =
                match inode_table.pending_buf_for_file_with_parent(ino, FILE_STR_PENDING_JSON)? {
                    Some(buf) => buf,
                    None => {
                        let bytes = default_bindings
                            .to_vec_pretty()
                            .map_err(|err| SiFileSystemError::Serialization(err.to_string()))?;

                        Arc::new(Cursor::new(bytes))
                    }
                };

            let ino = inode_table.upsert_with_parent_ino(
                entry.ino,
                FILE_STR_PENDING_JSON,
                InodeEntryData::SchemaFuncBindingsPending {
                    change_set_id,
                    schema_id,
                    kind,
                    buf: buf.clone(),
                    pending_func_id,
                },
                FileType::RegularFile,
                true,
                Size::UseExisting(buf.get_ref().len() as u64),
            )?;

            dirs.add(ino, FILE_STR_PENDING_JSON.into(), FileType::Directory);
        } else {
            if let Some(locked_id) = locked_id {
                let ino = inode_table.upsert_with_parent_ino(
                    entry.ino,
                    DIR_STR_LOCKED,
                    InodeEntryData::SchemaFuncDir {
                        kind,
                        func_id: locked_id,
                        change_set_id,
                        schema_id,
                        size: locked_size,
                        bindings_size: locked_binding_size,
                        types_size: locked_types_size,
                        unlocked: false,
                    },
                    FileType::Directory,
                    false,
                    Size::Directory,
                )?;
                dirs.add(ino, DIR_STR_LOCKED.into(), FileType::Directory);
            }

            if let Some(unlocked_id) = unlocked_id {
                let ino = inode_table.upsert_with_parent_ino(
                    entry.ino,
                    DIR_STR_UNLOCKED,
                    InodeEntryData::SchemaFuncDir {
                        kind,
                        func_id: unlocked_id,
                        change_set_id,
                        schema_id,
                        size: unlocked_size,
                        bindings_size: unlocked_bindings_size,
                        types_size: unlocked_types_size,
                        unlocked: true,
                    },
                    FileType::Directory,
                    false,
                    Size::Directory,
                )?;
                dirs.add(ino, DIR_STR_UNLOCKED.into(), FileType::Directory);
            }
        }

        Ok(())
    }

    async fn upsert_schema_func_kind_dir(
        &self,
        change_set_id: &ChangeSetId,
        schema_id: &SchemaId,
        kind: &FuncKind,
        entry: &InodeEntry,
        dirs: &mut DirListing,
    ) -> SiFileSystemResult<()> {
        let funcs_of_kind = self
            .client
            .variant_funcs_of_kind(*change_set_id, *schema_id, *kind)
            .await?;

        let mut existing_entries: HashMap<Inode, InodeEntry> = self
            .inode_table
            .direct_child_entries(entry.ino)?
            .into_iter()
            .map(|entry| (entry.ino, entry))
            .collect();

        for (func_name, funcs) in funcs_of_kind {
            let inode_table = &self.inode_table;

            let ino = inode_table.upsert_with_parent_ino(
                entry.ino,
                &func_name,
                InodeEntryData::SchemaFuncVariantsDir {
                    kind: *kind,
                    locked_id: funcs.locked.as_ref().map(|f| f.id),
                    unlocked_id: funcs.unlocked.as_ref().map(|f| f.id),
                    locked_size: funcs.locked.as_ref().map(|f| f.code_size).unwrap_or(0),
                    unlocked_size: funcs.unlocked.as_ref().map(|f| f.code_size).unwrap_or(0),
                    locked_types_size: funcs.locked.as_ref().map(|f| f.types_size).unwrap_or(0),
                    schema_id: *schema_id,
                    locked_bindings_size: funcs
                        .locked
                        .as_ref()
                        .map(|f| f.bindings_size)
                        .unwrap_or(0),
                    unlocked_bindings_size: funcs
                        .unlocked
                        .as_ref()
                        .map(|f| f.bindings_size)
                        .unwrap_or(0),
                    unlocked_types_size: funcs.unlocked.as_ref().map(|f| f.types_size).unwrap_or(0),
                    change_set_id: *change_set_id,
                    pending: false,
                    pending_func_id: None,
                },
                FileType::Directory,
                true,
                Size::Directory,
            )?;
            existing_entries.remove(&ino);
            dirs.add(ino, func_name, FileType::Directory);
        }

        for (ino, entry) in existing_entries {
            if matches!(
                entry.data(),
                &InodeEntryData::SchemaFuncVariantsDir { pending: true, .. }
            ) {
                dirs.add(ino, entry.name, entry.kind);
            }
        }

        Ok(())
    }

    async fn upsert_schema_funcs_dir(
        &self,
        entry: &InodeEntry,
        schema_id: &SchemaId,
        change_set_id: &ChangeSetId,
        dirs: &mut DirListing,
    ) -> SiFileSystemResult<()> {
        for kind in [
            FuncKind::Action,
            FuncKind::Attribute,
            FuncKind::Authentication,
            FuncKind::CodeGeneration,
            FuncKind::Management,
            FuncKind::Qualification,
        ] {
            let kind_pluralize_str = kind_pluralized_to_string(kind);
            let ino = self.inode_table.upsert_with_parent_ino(
                entry.ino,
                &kind_pluralize_str,
                InodeEntryData::SchemaFuncKindDir {
                    kind,
                    schema_id: *schema_id,
                    change_set_id: *change_set_id,
                },
                FileType::Directory,
                true,
                Size::Directory,
            )?;
            dirs.add(ino, kind_pluralize_str, FileType::Directory);
        }
        Ok(())
    }

    async fn upsert_schema_def_dir(
        &self,
        change_set_id: &ChangeSetId,
        schema_id: &SchemaId,
        entry: &InodeEntry,
        dirs: &mut DirListing,
    ) -> SiFileSystemResult<()> {
        let asset_funcs = self
            .client
            .asset_funcs_for_schema(*change_set_id, *schema_id)
            .await?;
        if let Some(unlocked_asset_func) = asset_funcs.unlocked {
            let ino = self.inode_table.upsert_with_parent_ino(
                entry.ino,
                DIR_STR_UNLOCKED,
                InodeEntryData::AssetDefinitionDir {
                    schema_id: *schema_id,
                    change_set_id: *change_set_id,
                    size: unlocked_asset_func.code_size,
                    attrs_size: asset_funcs.unlocked_attrs_size,
                    bindings_size: asset_funcs.unlocked_bindings_size,
                    types_size: asset_funcs.types_size,
                    unlocked: true,
                },
                FileType::Directory,
                false,
                Size::Directory,
            )?;
            dirs.add(ino, DIR_STR_UNLOCKED.into(), FileType::Directory);
        }
        if let Some(locked_asset_func) = asset_funcs.locked {
            let ino = self.inode_table.upsert_with_parent_ino(
                entry.ino,
                DIR_STR_LOCKED,
                InodeEntryData::AssetDefinitionDir {
                    schema_id: *schema_id,
                    change_set_id: *change_set_id,
                    size: locked_asset_func.code_size,
                    attrs_size: asset_funcs.locked_attrs_size,
                    bindings_size: asset_funcs.unlocked_bindings_size,
                    types_size: asset_funcs.types_size,
                    unlocked: false,
                },
                FileType::Directory,
                false,
                Size::Directory,
            )?;
            dirs.add(ino, DIR_STR_LOCKED.into(), FileType::Directory);
        };
        Ok(())
    }

    async fn upsert_schema_category_dir(
        &self,
        entry_ino: Inode,
        change_set_id: ChangeSetId,
        category: &str,
        dirs: &mut DirListing,
    ) -> SiFileSystemResult<()> {
        let schemas = self
            .client
            .list_schemas(change_set_id, Some(category))
            .await?;

        for schema in schemas {
            let ino = self.inode_table.upsert_with_parent_ino(
                entry_ino,
                &schema.name,
                InodeEntryData::SchemaDir {
                    schema_id: schema.id,
                    change_set_id,
                    name: schema.name.to_string(),
                    installed: schema.installed,
                },
                FileType::Directory,
                true,
                Size::Directory,
            )?;

            dirs.add(ino, schema.name.to_string(), FileType::Directory);
        }

        Ok(())
    }

    async fn upsert_schema_dir(
        &self,
        installed: &bool,
        entry: &InodeEntry,
        schema_id: &SchemaId,
        change_set_id: &ChangeSetId,
        dirs: &mut DirListing,
    ) -> SiFileSystemResult<()> {
        if *installed {
            let inode_table = &self.inode_table;
            let functions_ino = inode_table.upsert_with_parent_ino(
                entry.ino,
                DIR_STR_FUNCTIONS,
                InodeEntryData::SchemaFuncsDir {
                    schema_id: *schema_id,
                    change_set_id: *change_set_id,
                },
                FileType::Directory,
                true,
                Size::Directory,
            )?;
            dirs.add(functions_ino, DIR_STR_FUNCTIONS.into(), FileType::Directory);

            // add definition directory
            let schema_def_info = inode_table.upsert_with_parent_ino(
                entry.ino,
                DIR_STR_DEFINITION,
                InodeEntryData::SchemaDefinitionsDir {
                    schema_id: *schema_id,
                    change_set_id: *change_set_id,
                },
                FileType::Directory,
                true,
                Size::Directory,
            )?;

            dirs.add(
                schema_def_info,
                DIR_STR_DEFINITION.into(),
                FileType::Directory,
            );

            let installed_path = inode_table.make_path(Some(entry.ino), FILE_STR_INSTALLED)?;
            let installed_ino = match inode_table.ino_for_path(&installed_path) {
                Some(ino) => ino,
                None => inode_table.upsert_with_parent_ino(
                    entry.ino,
                    FILE_STR_INSTALLED,
                    InodeEntryData::InstalledSchemaMarker,
                    FileType::RegularFile,
                    false,
                    Size::Force(0),
                )?,
            };

            // "installed"  marker
            dirs.add(
                installed_ino,
                FILE_STR_INSTALLED.into(),
                FileType::RegularFile,
            );
        };
        Ok(())
    }

    async fn upsert_schemas_dir(
        &self,
        change_set_id: &ChangeSetId,
        entry: &InodeEntry,
        dirs: &mut DirListing,
    ) -> SiFileSystemResult<()> {
        let mut pending: HashMap<Inode, InodeEntry> = self
            .inode_table
            .direct_child_entries(entry.ino)?
            .into_iter()
            .filter(|entry| {
                matches!(
                    entry.data(),
                    InodeEntryData::SchemaCategoryDir { pending: true, .. }
                )
            })
            .map(|entry| (entry.ino, entry))
            .collect();

        let categories = self.client.list_schema_categories(*change_set_id).await?;
        for cat in categories {
            let cat_name = if cat.trim().is_empty() {
                DIR_STR_BLANK_CATEGORY
            } else {
                &cat
            };

            let cat_ino = self.upsert_category_dir(&cat, cat_name, entry.ino, *change_set_id)?;
            dirs.add(cat_ino, cat_name.to_string(), FileType::Directory);

            pending.remove(&cat_ino);
        }

        for (_, pending_cat) in pending {
            dirs.add(pending_cat.ino, pending_cat.name, FileType::Directory);
        }

        Ok(())
    }

    fn upsert_category_dir(
        &self,
        category: &str,
        cat_name: &str,
        schemas_ino: Inode,
        change_set_id: ChangeSetId,
    ) -> Result<Inode, SiFileSystemError> {
        let cat_ino = self.inode_table.upsert_with_parent_ino(
            schemas_ino,
            cat_name,
            InodeEntryData::SchemaCategoryDir {
                change_set_id,
                category: category.to_string(),
                pending: false,
            },
            FileType::Directory,
            true,
            Size::Directory,
        )?;

        Ok(cat_ino)
    }

    #[allow(clippy::too_many_arguments)]
    async fn upsert_change_set_func_dir(
        &self,
        entry: &InodeEntry,
        func_id: si_id::FuncId,
        change_set_id: ChangeSetId,
        size: u64,
        dirs: &mut DirListing,
        kind: FuncKind,
        types_size: u64,
    ) -> SiFileSystemResult<()> {
        let inode_table = &self.inode_table;
        let ino = inode_table.upsert_with_parent_ino(
            entry.ino,
            FILE_STR_TS_INDEX,
            InodeEntryData::FuncCode {
                func_id,
                change_set_id,
                kind,
            },
            FileType::RegularFile,
            false,
            Size::UseExisting(size),
        )?;
        dirs.add(ino, FILE_STR_TS_INDEX.into(), FileType::RegularFile);

        let ino = inode_table.upsert_with_parent_ino(
            entry.ino,
            FILE_STR_TS_INDEX_D_TS,
            InodeEntryData::FuncTypes {
                func_id,
                change_set_id,
            },
            FileType::RegularFile,
            false,
            Size::UseExisting(types_size),
        )?;
        dirs.add(ino, FILE_STR_TS_INDEX_D_TS.into(), FileType::RegularFile);

        let ino = inode_table.upsert_with_parent_ino(
            entry.ino,
            FILE_STR_DENO_CONFIG,
            InodeEntryData::FuncTypesDenoConfig,
            FileType::RegularFile,
            false,
            Size::UseExisting(DENO_CONFIG.len() as u64),
        )?;
        dirs.add(ino, FILE_STR_DENO_CONFIG.into(), FileType::RegularFile);

        Ok(())
    }

    async fn upsert_change_set_func_kind_dir(
        &self,
        change_set_id: &ChangeSetId,
        kind: &FuncKind,
        entry: &InodeEntry,
        dirs: &mut DirListing,
    ) -> SiFileSystemResult<()> {
        let funcs_of_kind = self
            .client
            .change_set_funcs_of_kind(*change_set_id, *kind)
            .await?;
        let mut names = HashSet::new();
        for func in funcs_of_kind {
            let func_name = if names.contains(&func.name) {
                format!("{}:{}", func.name, func.id)
            } else {
                names.insert(func.name.clone());
                func.name.clone()
            };

            let ino = self.inode_table.upsert_with_parent_ino(
                entry.ino,
                &func_name,
                InodeEntryData::ChangeSetFuncDir {
                    func_id: func.id,
                    change_set_id: *change_set_id,
                    size: func.code_size,
                    types_size: func.types_size,
                    kind: *kind,
                },
                FileType::Directory,
                true,
                Size::Directory,
            )?;
            dirs.add(ino, func_name, FileType::Directory);
        }
        Ok(())
    }

    async fn upsert_change_set_funcs_dir(
        &self,
        entry: &InodeEntry,
        change_set_id: &ChangeSetId,
        dirs: &mut DirListing,
    ) -> SiFileSystemResult<()> {
        for kind in [
            FuncKind::Action,
            FuncKind::Attribute,
            FuncKind::Authentication,
            FuncKind::CodeGeneration,
            FuncKind::Management,
            FuncKind::Qualification,
        ] {
            let kind_pluralize_str = kind_pluralized_to_string(kind);

            let ino = self.inode_table.upsert_with_parent_ino(
                entry.ino,
                &kind_pluralize_str,
                InodeEntryData::ChangeSetFuncKindDir {
                    kind,
                    change_set_id: *change_set_id,
                },
                FileType::Directory,
                true,
                Size::Directory,
            )?;
            dirs.add(ino, kind_pluralize_str, FileType::Directory);
        }
        Ok(())
    }

    async fn upsert_change_set_dir(
        &self,
        entry: &InodeEntry,
        change_set_id: &ChangeSetId,
        dirs: &mut DirListing,
    ) -> SiFileSystemResult<()> {
        let inode_table = &self.inode_table;

        let functions_ino = inode_table.upsert_with_parent_ino(
            entry.ino,
            DIR_STR_FUNCTIONS,
            InodeEntryData::ChangeSetFuncsDir {
                change_set_id: *change_set_id,
            },
            FileType::Directory,
            true,
            Size::Directory,
        )?;
        dirs.add(functions_ino, DIR_STR_FUNCTIONS.into(), FileType::Directory);
        let schemas_ino = inode_table.upsert_with_parent_ino(
            entry.ino,
            DIR_STR_SCHEMAS,
            InodeEntryData::SchemasDir {
                change_set_id: *change_set_id,
            },
            FileType::Directory,
            true,
            Size::Directory,
        )?;
        dirs.add(schemas_ino, DIR_STR_SCHEMAS.into(), FileType::Directory);
        Ok(())
    }

    async fn upsert_workspace_root(
        &self,
        entry: &InodeEntry,
        dirs: &mut DirListing,
    ) -> SiFileSystemResult<()> {
        let ino = self.inode_table.upsert_with_parent_ino(
            entry.ino,
            DIR_STR_CHANGE_SETS,
            InodeEntryData::ChangeSets,
            FileType::Directory,
            true,
            Size::Directory,
        )?;
        dirs.add(ino, DIR_STR_CHANGE_SETS.into(), FileType::Directory);

        let ino = self.inode_table.upsert_with_parent_ino(
            entry.ino,
            DIR_STR_VSCODE,
            InodeEntryData::VsCodeDir,
            FileType::Directory,
            false,
            Size::Directory,
        )?;
        dirs.add(ino, DIR_STR_VSCODE.into(), FileType::Directory);

        let ino = self.inode_table.upsert_with_parent_ino(
            entry.ino,
            FILE_STR_DENO_CONFIG,
            InodeEntryData::WorkspaceDenoJson,
            FileType::RegularFile,
            false,
            Size::UseExisting(0),
        )?;
        dirs.add(ino, FILE_STR_DENO_CONFIG.into(), FileType::RegularFile);

        Ok(())
    }

    async fn upsert_change_sets_dir(
        &self,
        ino: Inode,
        dirs: &mut DirListing,
    ) -> SiFileSystemResult<()> {
        let change_sets = self.client.list_change_sets().await?;
        for change_set in change_sets {
            let file_name = &change_set.name;
            let ino = self.inode_table.upsert_with_parent_ino(
                ino,
                file_name,
                InodeEntryData::ChangeSet {
                    change_set_id: change_set.id,
                    name: file_name.to_owned(),
                },
                FileType::Directory,
                true,
                Size::Directory,
            )?;

            dirs.add(ino, file_name.to_owned(), FileType::Directory);
        }
        Ok(())
    }

    async fn readdir(
        &self,
        ino: Inode,
        _fh: FileHandle,
        offset: i64,
        mut reply: ReplyDirectory,
    ) -> SiFileSystemResult<()> {
        if self.inode_table.entry_for_ino(ino).is_none() {
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

    async fn invalidate_change_set(&self, inactive_change_set_id: ChangeSetId) {
        let ino = self
            .inode_table
            .entries_by_path()
            .iter()
            .find(|entry_ref| {
                if let InodeEntryData::ChangeSet { change_set_id, .. } = entry_ref.data() {
                    *change_set_id == inactive_change_set_id
                } else {
                    false
                }
            })
            .map(|entry_ref| entry_ref.ino);

        if let Some(ino) = ino {
            self.inode_table.invalidate_ino(ino);
        }
    }

    async fn command_handler_loop(&mut self, mut rx: UnboundedReceiver<FilesystemCommand>) {
        while let Some(command) = rx.recv().await {
            let self_clone = self.clone();

            // Block here while we hydrate
            if let FilesystemCommand::HydrateChangeSets = &command {
                if let Err(err) = self.hydrate().await {
                    log::error!("failed to hydrate: {err:?}");
                }

                continue;
            }

            tokio::task::spawn(async move {
                let res = match command {
                    FilesystemCommand::HydrateChangeSets => {
                        log::error!("Should be unreachable");
                        Ok(())
                    }
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
                    FilesystemCommand::FSync { reply, .. } => {
                        reply.ok();
                        Ok(())
                    }
                    FilesystemCommand::GetXattr { reply, .. } => {
                        reply.error(ENOSYS);
                        Ok(())
                    }
                    FilesystemCommand::SetXattr { reply, .. } => {
                        reply.error(ENOSYS);
                        Ok(())
                    }
                    FilesystemCommand::Flush { reply, .. } => {
                        reply.ok();
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
                        ino,
                        fh,
                        offset,
                        data,
                        write_flags,
                        flags,
                        lock_owner,
                        reply,
                    } => {
                        self_clone
                            .write(ino, fh, offset, data, write_flags, flags, lock_owner, reply)
                            .await
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
                    FilesystemCommand::Create {
                        parent,
                        name,
                        mode,
                        umask,
                        flags,
                        reply,
                    } => {
                        self_clone
                            .create(parent, name, mode, umask, flags, reply)
                            .await
                    }
                    FilesystemCommand::Forget { .. } => Ok(()),
                    FilesystemCommand::Rename {
                        parent,
                        name,
                        newparent,
                        newname,
                        flags,
                        reply,
                    } => {
                        self_clone
                            .rename(parent, name, newparent, newname, flags, reply)
                            .await
                    }
                    command => {
                        dbg!(&command);
                        command.error(ENOSYS);
                        Ok(())
                    }
                };

                if let Err(err) = res {
                    if let SiFileSystemError::SiFsClient(SiFsClientError::BackendError(
                        FsApiError::ChangeSetInactive(change_set_id),
                    )) = err
                    {
                        log::trace!("invaliding changeset {change_set_id}");
                        self_clone.invalidate_change_set(change_set_id).await;
                    } else {
                        log::error!("{err:?}");
                    }
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

    let mount_point = mount_point.as_ref();
    let mount_point_clone = mount_point.to_path_buf();

    runtime_handle.spawn(async move {
        SiFileSystem::new(mount_point_clone, token, endpoint, workspace_id, uid, gid)
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

fn default_attribute_bindings() -> Binding {
    Binding::Attribute {
        output_to: AttributeOutputTo::Prop(
            "CHOOSE AN OUTPUT LOCATION like: \"root/si/name\"".into(),
        ),
        inputs: BTreeMap::new(),
    }
}

fn default_action_bindings() -> Binding {
    Binding::Action {
        kind: ActionKind::Create,
    }
}
