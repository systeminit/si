use std::{
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

use petgraph::prelude::*;
use tar::Builder;
use thiserror::Error;
use tokio::{fs::File, task};
use vfs::{PhysicalFS, VfsError, VfsPath};
use vfs_tar::TarFS;

use crate::{
    graph::{
        GraphError, HashedNodeWithEntries, NameStr, NodeEntry, NodeWithEntries, ObjectTree,
        ReadBytes, WriteBytes,
    },
    hash::{Hash, HashParseError},
};

const ROOT_DIRS: &[&str] = &["refs", "objects"];

/// An error that can be returned when working with a file system reader or writer.
#[remain::sorted]
#[derive(Debug, Error)]
pub enum FsError {
    /// When a file fails to be created
    #[error("error when creating file: {0}")]
    Create(#[source] VfsError),
    /// When an error is returned while writing a serialized node representation
    #[error("failed write bytes: {0}")]
    GraphWrite(#[source] GraphError),
    /// When a hash value fails to be parsed
    #[error(transparent)]
    HashParse(#[from] HashParseError),
    /// When an invalid path is found
    #[error("invalid path: {0}")]
    InvalidPath(#[source] VfsError),
    /// When an error occurs while reading from a file system
    #[error("io error when reading from filesystem: {0}")]
    IoRead(#[source] io::Error),
    /// When an error occurs while writing to a file system
    #[error("io error when writing to filesystem: {0}")]
    IoWrite(#[source] io::Error),
    /// When a Tokio blocking task fails to execute to completion
    #[error("blocking task failed to execute to completion")]
    Join,
    /// When a filename has no basename entry
    #[error("basename could not be determined for path: {0}")]
    NoBasename(PathBuf),
    /// When failing to parse a node with entries from bytes
    #[error("failed parse node with entries from bytes: {0}")]
    NodeWithEntriesParse(#[source] GraphError),
    /// When failing to open a file for reading
    #[error("error when opening file for read: {0}")]
    OpenRead(#[source] VfsError),
    /// When failing while reading a tree
    #[error("error when reading tree: {0}")]
    ReadTree(#[source] GraphError),
    /// When failing to create a temporary directory
    #[error("error when creating temp dir: {0}")]
    TempDir(#[source] io::Error),
}

/// Reads an [`ObjectTree`] from a file system.
#[remain::sorted]
#[derive(Clone, Debug)]
pub enum TreeFileSystemReader {
    /// A reader rooted in a physical file system
    Physical {
        /// Virtual file system root
        vfs_path: VfsPath,
        /// Base path to physical file system
        base_path: PathBuf,
    },
    /// A reader from a Tar file
    Tar {
        /// Virtual file system root
        vfs_path: VfsPath,
        /// Path to Tar file
        tar_file: PathBuf,
    },
}

impl TreeFileSystemReader {
    /// Creates a reader rooted in a physical file system.
    pub fn physical(base_path: impl Into<PathBuf>) -> Self {
        let base_path = base_path.into();
        let fs = PhysicalFS::new(&base_path);
        let vfs_path = VfsPath::new(fs);

        Self::Physical {
            vfs_path,
            base_path,
        }
    }

    /// Creates a reader from a Tar file.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the Tar file does not exist or cannot be opened
    pub async fn tar(tar_file: impl Into<PathBuf>) -> Result<Self, FsError> {
        let tar_file = tar_file.into();
        let file = File::open(&tar_file)
            .await
            .map_err(FsError::IoRead)?
            .into_std()
            .await;
        let fs = asyncify(move || TarFS::from_std_file(&file).map_err(FsError::OpenRead)).await?;
        let vfs_path = VfsPath::new(fs);

        Ok(Self::Tar { vfs_path, tar_file })
    }

    /// Reads and returns an [`ObjectTree`] from the underlying file system.
    ///
    /// # Errors
    ///
    /// Returns `Err` if:
    ///
    /// - An I/O error occurs while reading from a file
    /// - An expected file does not exist or cannot be opened
    /// - A node file fails to be correctly parsed
    /// - The resulting tree structure has no root node or multiple root nodes
    pub async fn read<T>(self) -> Result<ObjectTree<T>, FsError>
    where
        T: ReadBytes,
    {
        let mut graph = Graph::new();
        let mut root_idx: Option<NodeIndex> = None;

        let root_hash = self.read_root_ref().await?;
        let root_node = self.read_node(root_hash).await?;

        let mut stack: Vec<(HashedNodeWithEntries<T>, Option<NodeIndex>)> = vec![(root_node, None)];

        while let Some((node_with_entries, parent_idx)) = stack.pop() {
            let (node, child_entries) = node_with_entries.into();

            let node_idx = graph.add_node(node);

            match parent_idx {
                Some(parent_idx) => {
                    graph.add_edge(parent_idx, node_idx, ());
                }
                None => match root_idx {
                    None => {
                        root_idx = Some(node_idx);
                    }
                    Some(_) => return Err(FsError::ReadTree(GraphError::MultipleRootNode)),
                },
            };

            for child_entry in child_entries.into_iter().rev() {
                let child_node = self.read_node(child_entry.hash()).await?;
                stack.push((child_node, Some(node_idx)));
            }
        }

        match root_idx {
            Some(root_idx) => Ok(ObjectTree::new(graph, root_idx)),
            None => Err(FsError::ReadTree(GraphError::MissingRootNode)),
        }
    }

    fn vfs_path(&self) -> &VfsPath {
        match self {
            Self::Physical { vfs_path, .. } | Self::Tar { vfs_path, .. } => vfs_path,
        }
    }

    fn object_path(&self, hash: Hash) -> Result<VfsPath, FsError> {
        object_path(self.vfs_path(), hash)
    }

    fn ref_path(&self, name: impl AsRef<str>) -> Result<VfsPath, FsError> {
        ref_path(self.vfs_path(), name)
    }

    async fn read_node<T>(&self, hash: Hash) -> Result<HashedNodeWithEntries<T>, FsError>
    where
        T: ReadBytes,
    {
        let dst_path = self.object_path(hash)?;
        let mut buf = Vec::new();

        let buf = asyncify(move || {
            let mut f = dst_path.open_file().map_err(FsError::OpenRead)?;
            f.read_to_end(&mut buf).map_err(FsError::IoRead)?;
            Ok(buf)
        })
        .await?;

        let node_with_entries: NodeWithEntries<T> =
            NodeWithEntries::from_bytes(buf).map_err(FsError::NodeWithEntriesParse)?;

        Ok(HashedNodeWithEntries::from_node_with_entries_and_hash(
            node_with_entries,
            hash,
        ))
    }

    async fn read_root_ref(&self) -> Result<Hash, FsError> {
        let dst_path = self.ref_path("root")?;
        let mut buf = String::new();

        let buf = asyncify(move || {
            let mut f = dst_path.open_file().map_err(FsError::OpenRead)?;
            f.read_to_string(&mut buf).map_err(FsError::IoRead)?;
            Ok(buf)
        })
        .await?;

        Hash::from_str(&buf).map_err(Into::into)
    }
}

/// Writes and [`ObjectTree`] to a file system.
#[remain::sorted]
#[derive(Clone, Debug)]
pub enum TreeFileSystemWriter {
    /// A writer rooted in a physical file system
    Physical {
        /// Virtual file system root
        vfs_path: VfsPath,
    },
    /// A writer to a Tar file
    Tar {
        /// Path to Tar file
        tar_file: PathBuf,
    },
}

impl TreeFileSystemWriter {
    /// Creates a writer rooted in a physical file system.
    pub fn physical(base_path: impl AsRef<Path>) -> Self {
        let vfs_path = VfsPath::new(PhysicalFS::new(base_path));

        Self::Physical { vfs_path }
    }

    /// Creates a writer to a Tar file.
    pub async fn tar(tar_file: impl Into<PathBuf>) -> Result<Self, FsError> {
        let tar_file = tar_file.into();

        Ok(Self::Tar { tar_file })
    }

    /// Writes an [`ObjectTree`] to the target file system.
    ///
    /// # Errors
    ///
    /// Returns `Err` if:
    ///
    /// - An I/O error occurs
    /// - A file fails to be created or opened for writing
    /// - A directory can't be created
    /// - A node fails to properly serialize to a file
    pub async fn write<T>(self, tree: &ObjectTree<T>) -> Result<(), FsError>
    where
        T: Clone + NameStr + WriteBytes + Send + Sync + 'static,
    {
        match self {
            Self::Physical { vfs_path } => {
                PhysicalFileSystemWriter::new(&vfs_path).write(tree).await
            }
            Self::Tar { tar_file } => TarFileSystemWriter::new(&tar_file).write(tree).await,
        }
    }
}

/// Writes an [`ObjectTree`] to a physical file system.
struct PhysicalFileSystemWriter<'a> {
    vfs_path: &'a VfsPath,
}

impl<'a> PhysicalFileSystemWriter<'a> {
    fn new(vfs_path: &'a VfsPath) -> Self {
        Self { vfs_path }
    }

    async fn write<T>(&self, tree: &ObjectTree<T>) -> Result<(), FsError>
    where
        T: Clone + NameStr + WriteBytes + Send + Sync + 'static,
    {
        let (graph, root_idx) = tree.as_petgraph();

        self.create_tree_dirs().await?;

        let mut dfspo = DfsPostOrder::new(graph, root_idx);

        while let Some(node_idx) = dfspo.next(graph) {
            let node = graph[node_idx].clone();

            // Create an entry for each direct child
            let mut entries = Vec::new();
            for child_idx in graph.neighbors_directed(node_idx, Outgoing) {
                let child_node = &graph[child_idx];
                entries.push(NodeEntry::new(
                    child_node.kind(),
                    child_node.hash(),
                    child_node.name(),
                ));
            }

            self.write_node(HashedNodeWithEntries::new(node, entries))
                .await?;
        }

        let root_hash = graph[root_idx].hash();
        self.write_root_ref(root_hash).await?;

        Ok(())
    }

    async fn create_tree_dirs(&self) -> Result<(), FsError> {
        let dirs = root_vfs_paths(self.vfs_path)?;

        asyncify(move || {
            for dir in dirs {
                dir.create_dir_all().map_err(FsError::InvalidPath)?;
            }
            Ok(())
        })
        .await?;

        Ok(())
    }

    async fn write_node<T>(&self, node: HashedNodeWithEntries<T>) -> Result<(), FsError>
    where
        T: WriteBytes + Send + Sync + 'static,
    {
        let dst_path = self.object_path(node.hash())?;
        let buf = node.to_bytes().map_err(FsError::GraphWrite)?;

        asyncify(move || {
            let f = dst_path.create_file().map_err(FsError::Create)?;
            let mut writer = BufWriter::new(f);
            writer.write_all(&buf).map_err(FsError::IoWrite)
        })
        .await
    }

    async fn write_root_ref(&self, hash: Hash) -> Result<(), FsError> {
        let dst_path = self.ref_path("root")?;
        let buf = hash.to_string();

        asyncify(move || {
            let f = dst_path.create_file().map_err(FsError::Create)?;
            let mut writer = BufWriter::new(f);
            writer.write_all(buf.as_bytes()).map_err(FsError::IoWrite)
        })
        .await
    }

    fn object_path(&self, hash: Hash) -> Result<VfsPath, FsError> {
        object_path(self.vfs_path, hash)
    }

    fn ref_path(&self, name: impl AsRef<str>) -> Result<VfsPath, FsError> {
        ref_path(self.vfs_path, name)
    }
}

/// Writes an [`ObjectTree`] to a Tar file.
struct TarFileSystemWriter<'a> {
    tar_file: &'a Path,
}

impl<'a> TarFileSystemWriter<'a> {
    fn new(tar_file: &'a Path) -> Self {
        Self { tar_file }
    }

    async fn write<T>(&self, tree: &ObjectTree<T>) -> Result<(), FsError>
    where
        T: Clone + NameStr + WriteBytes + Send + Sync + 'static,
    {
        // Write the tree to a physical fs in a tmpdir to get the on-disk structure
        let tmpdir = asyncify(move || tempfile::TempDir::new().map_err(FsError::TempDir)).await?;
        PhysicalFileSystemWriter::new(&VfsPath::new(PhysicalFS::new(tmpdir.path())))
            .write(tree)
            .await?;

        // Create a tarball with the root paths of the tmpdir as root directory entries
        let file = File::create(self.tar_file)
            .await
            .map_err(FsError::IoWrite)?;
        let mut builder = Builder::new(file.into_std().await);
        let dirs = root_paths(tmpdir.path()).into_iter();
        let _written_file = asyncify(move || {
            for dir in dirs {
                builder
                    .append_dir_all(
                        dir.file_name()
                            .ok_or_else(|| FsError::NoBasename(dir.clone()))?,
                        &dir,
                    )
                    .map_err(FsError::IoWrite)?;
            }

            // Ensure the tarball is finished and contents flushed to disk
            builder.into_inner().map_err(FsError::IoWrite)
        })
        .await?;

        Ok(())
    }
}

// Adapted from Tokio's internal `asyncify` which is the underlying impl for all `tokio::fs` calls.
//
// See: https://github.com/tokio-rs/tokio/blob/74fb9e387aa3604193ac7da0b103c96ff90c73ee/tokio/src/fs/mod.rs#L132-L144
async fn asyncify<F, T>(f: F) -> Result<T, FsError>
where
    F: FnOnce() -> Result<T, FsError> + Send + 'static,
    T: Send + 'static,
{
    match task::spawn_blocking(f).await {
        Ok(res) => res,
        Err(_join_err) => Err(FsError::Join),
    }
}

fn object_path(vfs_path: &VfsPath, hash: Hash) -> Result<VfsPath, FsError> {
    vfs_path
        .join("objects")
        .map_err(FsError::InvalidPath)?
        .join(hash.to_string())
        .map_err(FsError::InvalidPath)
}

fn ref_path(vfs_path: &VfsPath, name: impl AsRef<str>) -> Result<VfsPath, FsError> {
    vfs_path
        .join("refs")
        .map_err(FsError::InvalidPath)?
        .join(name)
        .map_err(FsError::InvalidPath)
}

fn root_vfs_paths(base_path: &VfsPath) -> Result<Vec<VfsPath>, FsError> {
    ROOT_DIRS
        .iter()
        .map(|dir| base_path.join(dir).map_err(FsError::InvalidPath))
        .collect()
}

fn root_paths(base_path: &Path) -> Vec<PathBuf> {
    ROOT_DIRS.iter().map(|dir| base_path.join(dir)).collect()
}
