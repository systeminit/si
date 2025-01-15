use std::ffi::OsString;
use std::path::Path;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;

use client::{SiFsClient, SiFsClientError};
use fuser::{FileType, MountOption, ReplyAttr, ReplyDirectory, ReplyEntry, ReplyOpen};
use inode_table::{InodeEntryData, InodeTable, InodeTableError};
use nix::unistd::{Gid, Uid};
use nix::{
    libc::{ENODATA, ENOENT, ENOSYS},
    unistd,
};
use thiserror::Error;
use tokio::runtime::{self};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::RwLock;

pub use si_id::WorkspaceId;

mod async_wrapper;
mod client;
mod command;
mod inode_table;

pub use async_wrapper::AsyncFuseWrapper;
pub use command::FilesystemCommand;

const FILE_HANDLE_READ_BIT: u64 = 1 << 63;
// const FILE_HANDLE_WRITE_BIT: u64 = 1 << 62;

#[derive(Error, Debug)]
pub enum SiFileSystemError {
    #[error("inode table error: {0}")]
    InodeTable(#[from] InodeTableError),
    #[error("si-fs client error: {0}")]
    SiFsClient(#[from] SiFsClientError),
}

pub type SiFileSystemResult<T> = Result<T, SiFileSystemError>;

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct SiFileSystem {
    client: Arc<SiFsClient>,
    workspace_id: WorkspaceId,
    inode_table: Arc<RwLock<InodeTable>>,
    fh: Arc<AtomicU64>,
    uid: Uid,
    gid: Gid,
}

const TTL: Duration = Duration::from_secs(0);

struct DirEntry {
    ino: u64,
    name: String,
    kind: FileType,
}

struct DirListing {
    entries: Vec<DirEntry>,
}

impl DirListing {
    pub fn new(ino: u64, parent: Option<u64>) -> Self {
        let entries = vec![
            DirEntry {
                ino,
                name: ".".into(),
                kind: FileType::Directory,
            },
            DirEntry {
                ino: parent.unwrap_or(1),
                name: "..".into(),
                kind: FileType::Directory,
            },
        ];

        Self { entries }
    }

    pub fn add(&mut self, ino: u64, name: String, kind: FileType) {
        self.entries.push(DirEntry { ino, name, kind });
    }

    pub fn send_reply(&self, reply: &mut ReplyDirectory, offset: i64) {
        for (i, entry) in self.entries.iter().enumerate().skip(offset as usize) {
            if reply.add(entry.ino, (i + 1) as i64, entry.kind, &entry.name) {
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
            fh: Arc::new(AtomicU64::new(1)),
            uid,
            gid,
        }
    }

    fn get_file_handle(&self) -> u64 {
        self.fh.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    async fn getattr(
        &self,
        ino: u64,
        _fh: Option<u64>,
        reply: ReplyAttr,
    ) -> SiFileSystemResult<()> {
        let Some(entry) = self.inode_table.read().await.get(ino).cloned() else {
            reply.error(ENOENT);
            return Ok(());
        };

        reply.attr(&TTL, entry.attrs());

        Ok(())
    }

    async fn open(&self, _ino: u64, reply: ReplyOpen, _flags: i32) -> SiFileSystemResult<()> {
        reply.opened(self.get_file_handle() | FILE_HANDLE_READ_BIT, 0);
        Ok(())
    }

    async fn opendir(&self, _ino: u64, reply: ReplyOpen, _flags: i32) -> SiFileSystemResult<()> {
        reply.opened(self.get_file_handle() | FILE_HANDLE_READ_BIT, 0);
        Ok(())
    }

    async fn mkdir(
        &self,
        parent: u64,
        name: OsString,
        _mode: u32,
        _umask: u32,
        reply: ReplyEntry,
    ) -> SiFileSystemResult<()> {
        let name = name.into_string().expect("received non utf8 name");

        let parent_entry = {
            let inode_table = self.inode_table.read().await;
            let Some(parent_entry) = inode_table.get(parent) else {
                reply.error(ENOENT);
                return Ok(());
            };

            parent_entry.to_owned()
        };

        match parent_entry.data() {
            InodeEntryData::WorkspaceRoot { .. } => {
                let change_set = self.client.create_change_set(name.to_owned()).await?;

                let attrs = {
                    let mut inode_table = self.inode_table.write().await;
                    let ino = inode_table.upsert_with_parent_ino(
                        parent,
                        &name,
                        InodeEntryData::ChangeSet {
                            id: change_set.id,
                            name: name.to_owned(),
                        },
                        FileType::Directory,
                    )?;
                    inode_table.make_attrs(ino, FileType::Directory, 0o755)
                };

                reply.entry(&TTL, &attrs, 1);
            }
            InodeEntryData::ChangeSet { .. } => {
                reply.error(ENOSYS);
            }
            InodeEntryData::Schema { .. } => {
                reply.error(ENOSYS);
            }
            InodeEntryData::SchemaVariant { .. } => {
                reply.error(ENOSYS);
            }
        }

        Ok(())
    }

    async fn lookup(
        &self,
        parent: u64,
        name: impl AsRef<Path>,
        reply: ReplyEntry,
    ) -> SiFileSystemResult<()> {
        let read_table = self.inode_table.read().await;

        let Some(parent_path) = read_table.path(parent) else {
            reply.error(ENOENT);
            return Ok(());
        };

        let full_path = parent_path.join(name);
        let Some(entry_ino) = read_table.ino_for_path(&full_path) else {
            reply.error(ENOENT);
            return Ok(());
        };

        if let Some(entry) = read_table.get(entry_ino) {
            reply.entry(&TTL, entry.attrs(), 0);
        } else {
            reply.error(ENOENT);
        }

        Ok(())
    }

    async fn readdir(
        &self,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) -> SiFileSystemResult<()> {
        let Some(entry) = self.inode_table.read().await.get(ino).cloned() else {
            reply.error(ENOENT);
            return Ok(());
        };

        let mut dirs = DirListing::new(ino, entry.parent);

        match entry.data() {
            InodeEntryData::WorkspaceRoot { .. } => {
                let change_sets = self.client.list_change_sets().await?;

                for change_set in change_sets {
                    let mut inode_table = self.inode_table.write().await;

                    let file_name = &change_set.name;
                    let ino = inode_table.upsert_with_parent_ino(
                        ino,
                        file_name,
                        InodeEntryData::ChangeSet {
                            id: change_set.id,
                            name: file_name.to_owned(),
                        },
                        FileType::Directory,
                    )?;

                    dirs.add(ino, file_name.to_owned(), FileType::Directory);
                }

                dirs.send_reply(&mut reply, offset);

                reply.ok();
            }
            InodeEntryData::ChangeSet { id, .. } => {
                let schemas = self
                    .client
                    .schemas(*id)
                    .await
                    .expect("failed to fetch variants");

                for schema in schemas {
                    let mut inode_table = self.inode_table.write().await;
                    let ino = inode_table.upsert_with_parent_ino(
                        entry.ino,
                        &schema.name,
                        InodeEntryData::Schema {
                            id: schema.id,
                            name: schema.name.clone(),
                            installed: schema.installed,
                            change_set_id: *id,
                        },
                        FileType::Directory,
                    )?;
                    dirs.add(ino, schema.name.clone(), FileType::Directory);
                }

                dirs.send_reply(&mut reply, offset);
                reply.ok();
            }
            InodeEntryData::Schema {
                id, change_set_id, ..
            } => {
                let variants = self.client.variants(*change_set_id, *id).await?;
                if let Some(unlocked_variant_id) = variants.unlocked {
                    let mut inode_table = self.inode_table.write().await;

                    let ino = inode_table.upsert_with_parent_ino(
                        entry.ino,
                        "unlocked",
                        InodeEntryData::SchemaVariant {
                            id: unlocked_variant_id,
                            schema_id: *id,
                            change_set_id: *change_set_id,
                            locked: false,
                        },
                        FileType::Directory,
                    )?;
                    dirs.add(ino, "unlocked".into(), FileType::Directory);
                }

                if let Some(unlocked_variant_id) = variants.locked {
                    let mut inode_table = self.inode_table.write().await;

                    let ino = inode_table.upsert_with_parent_ino(
                        entry.ino,
                        "locked",
                        InodeEntryData::SchemaVariant {
                            id: unlocked_variant_id,
                            schema_id: *id,
                            change_set_id: *change_set_id,
                            locked: false,
                        },
                        FileType::Directory,
                    )?;
                    dirs.add(ino, "locked".into(), FileType::Directory);
                }

                dirs.send_reply(&mut reply, offset);
                reply.ok();
            }
            InodeEntryData::SchemaVariant { .. } => {
                reply.error(ENOSYS);
            }
        }

        Ok(())
    }

    async fn command_handler_loop(&mut self, mut rx: UnboundedReceiver<FilesystemCommand>) {
        while let Some(command) = rx.recv().await {
            //dbg!(&command);
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
                    FilesystemCommand::Release { reply, .. } => {
                        reply.ok();
                        Ok(())
                    }
                    FilesystemCommand::GetXattr { reply, .. } => {
                        reply.error(ENODATA);
                        Ok(())
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
) {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let async_fuse_wrapper = AsyncFuseWrapper::new(tx);

    let uid = unistd::geteuid();
    let gid = unistd::getegid();

    runtime_handle.spawn(async move {
        SiFileSystem::new(token, endpoint, workspace_id, uid, gid)
            .command_handler_loop(rx)
            .await
    });

    let default_options = vec![
        MountOption::FSName("si-filesystem".to_string()),
        MountOption::AutoUnmount,
        MountOption::RW,
        MountOption::DefaultPermissions,
    ];

    let mut options = options.unwrap_or_default();

    options.extend_from_slice(&default_options);

    fuser::mount2(async_fuse_wrapper, mount_point, &options).expect("mount fuse fs");
}
