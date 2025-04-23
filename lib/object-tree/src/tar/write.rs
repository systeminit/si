use std::{
    num::TryFromIntError,
    path::PathBuf,
};

use ::tar::{
    Builder,
    Header,
};
use petgraph::prelude::*;
use thiserror::Error;

use crate::{
    GraphError,
    NameStr,
    ObjectTree,
    WriteBytes,
    graph::{
        HashedNodeWithEntries,
        NodeEntry,
    },
    tar::{
        object_path,
        ref_path,
    },
};

/// Errors that can occur when creating a tar bundle of the object tree
#[remain::sorted]
#[derive(Debug, Error)]
pub enum TarWriterError {
    /// When the object tree cannot be converted to a petgraph graph
    #[error("GraphError: {0}")]
    Graph(#[from] GraphError),
    /// When an entry cannot be added to the tar file
    #[error("IoError: {0}")]
    Io(#[from] std::io::Error),
    /// When the length of the entry is unable to be converted to the size entry of the tar header
    #[error("TryFromIntError: {0}")]
    TryFromInt(#[from] TryFromIntError),
}

/// Create a tar from an [`ObjectTree`]
pub struct TarWriter {
    bytes: Vec<u8>,
}

impl TarWriter {
    /// Return a [`TarWriter`] populated from the provided [`ObjectTree`]
    pub fn new<T>(tree: &ObjectTree<T>) -> Result<Self, TarWriterError>
    where
        T: Clone + NameStr + WriteBytes + Send + Sync + 'static,
    {
        let (graph, root_idx) = tree.as_petgraph();
        let mut tar_builder = Builder::new(Vec::new());

        let mut dfspo = DfsPostOrder::new(graph, root_idx);

        while let Some(node_idx) = dfspo.next(graph) {
            let node = graph
                .node_weight(node_idx)
                .ok_or(GraphError::NodeWeightNotFound(
                    node_idx.index(),
                    "tar writer: could not find next node for index for dfspo",
                ))?
                .clone();

            let mut entries = Vec::new();
            for child_idx in graph.neighbors_directed(node_idx, Outgoing) {
                let child_node =
                    graph
                        .node_weight(child_idx)
                        .ok_or(GraphError::NodeWeightNotFound(
                            child_idx.index(),
                            "tar writer: could not find child node for index",
                        ))?;
                entries.push(NodeEntry::new(
                    child_node.kind(),
                    child_node.hash(),
                    child_node.name(),
                ));
            }

            let tar_entry = HashedNodeWithEntries::new(node, entries);
            write_tar_entry(
                &mut tar_builder,
                object_path(&tar_entry.hash()),
                &tar_entry.to_bytes()?,
            )?;
        }

        let root_node = graph
            .node_weight(root_idx)
            .ok_or(GraphError::NodeWeightNotFound(
                root_idx.index(),
                "tar writer: could not find root node for index",
            ))?;
        write_tar_entry(
            &mut tar_builder,
            ref_path("root"),
            root_node.hash().to_string().as_bytes(),
        )?;
        tar_builder.finish()?;

        Ok(Self {
            bytes: tar_builder.into_inner()?,
        })
    }

    /// Return the tar as a `Vec<u8>`
    pub fn bytes(self) -> Vec<u8> {
        self.bytes
    }
}

fn write_tar_entry(
    tar_builder: &mut Builder<Vec<u8>>,
    path: PathBuf,
    entry: &[u8],
) -> Result<(), TarWriterError> {
    let mut tar_entry_header = Header::new_gnu();
    tar_entry_header.set_path(&path)?;
    tar_entry_header.set_size(entry.len().try_into()?);
    tar_entry_header.set_cksum();

    tar_builder.append(&tar_entry_header, entry)?;

    Ok(())
}
