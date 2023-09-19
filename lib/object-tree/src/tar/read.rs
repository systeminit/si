use std::{collections::HashMap, io::Read, path::PathBuf, str::FromStr, string::FromUtf8Error};

use petgraph::prelude::*;
use thiserror::Error;

use crate::{
    graph::{GraphError, HashedNodeWithEntries, NodeWithEntries, ObjectTree, ReadBytes},
    hash::{Hash, HashParseError},
    tar::{object_path, ref_path},
};

/// Errors that can occur when reading a module bundle from a tar file
#[derive(Debug, Error)]
pub enum TarReadError {
    /// When an error occurs creating a [`struct@Hash`] from the given
    /// [`String`]
    #[error("Error parsing hash: {0}")]
    Hash(#[from] HashParseError),
    /// When an error occurs while reading bytes
    #[error("io error when reading: {0}")]
    IoRead(#[from] std::io::Error),
    /// When the given entry is not found in what was read from the `tar`
    #[error("Node entry not found: {0:?}")]
    NodeNotFound(PathBuf),
    /// When failing to parse a node with entries from bytes
    #[error("failed to parse node with entries from bytes: {0}")]
    NodeWithEntriesParse(#[source] GraphError),
    /// When failing while reading a tree
    #[error("error when reading tree: {0}")]
    ReadTree(#[source] GraphError),
    /// When the root node is not recognized as a valid node kind
    #[error("could not read root node")]
    RootNodeError,
    /// When the given byte sequence is not parsable as a UTF8 [`String`]
    #[error("Invalid string: {0}")]
    StringParse(#[from] FromUtf8Error),
}

impl<T> ObjectTree<T> {
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
    pub fn read_from_tar<N>(tar_data: Vec<u8>) -> Result<ObjectTree<N>, TarReadError>
    where
        N: ReadBytes,
    {
        let mut graph = Graph::new();
        let mut root_idx: Option<NodeIndex> = None;

        let mut unpacked_tar = ::tar::Archive::new(tar_data.as_slice());
        let mut tar_data = HashMap::new();
        for maybe_tar_entry in unpacked_tar.entries()? {
            let mut tar_entry = maybe_tar_entry?;
            let entry_path = tar_entry.path()?.into_owned();
            let mut entry_data = Vec::new();
            tar_entry.read_to_end(&mut entry_data)?;

            tar_data.insert(entry_path, entry_data);
        }

        let root_hash = get_root_ref(&mut tar_data)?;
        let root_node = get_node(&mut tar_data, root_hash)?.ok_or(TarReadError::RootNodeError)?;

        let mut stack: Vec<(HashedNodeWithEntries<N>, Option<NodeIndex>)> = vec![(root_node, None)];

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
                    Some(_) => return Err(TarReadError::ReadTree(GraphError::MultipleRootNode)),
                },
            };

            for child_entry in child_entries.into_iter().rev() {
                if let Some(child_node) = get_node(&mut tar_data, child_entry.hash())? {
                    stack.push((child_node, Some(node_idx)));
                }
            }
        }

        match root_idx {
            Some(root_idx) => Ok(ObjectTree::new(graph, root_idx)),
            None => Err(TarReadError::ReadTree(GraphError::MissingRootNode)),
        }
    }
}

fn get_node<N>(
    tar_data: &mut HashMap<PathBuf, Vec<u8>>,
    hash: Hash,
) -> Result<Option<HashedNodeWithEntries<N>>, TarReadError>
where
    N: ReadBytes,
{
    let dst_path = object_path(&hash);
    let buf = tar_data
        .get(&dst_path)
        .ok_or_else(|| TarReadError::NodeNotFound(dst_path))?;

    let node_with_entries: Option<NodeWithEntries<N>> =
        NodeWithEntries::from_bytes(buf.clone()).map_err(TarReadError::NodeWithEntriesParse)?;

    Ok(node_with_entries
        .map(|nwe| HashedNodeWithEntries::from_node_with_entries_and_hash(nwe, hash)))
}

fn get_root_ref(tar_data: &mut HashMap<PathBuf, Vec<u8>>) -> Result<Hash, TarReadError> {
    let dst_path = ref_path("root");
    let buf = String::from_utf8(
        tar_data
            .get(&dst_path)
            .cloned()
            .ok_or(TarReadError::NodeNotFound(dst_path))?,
    )?;

    Hash::from_str(&buf).map_err(Into::into)
}
