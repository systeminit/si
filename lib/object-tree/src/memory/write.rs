use std::{
    num::TryFromIntError,
    path::{Path, PathBuf},
};

use petgraph::prelude::*;
use tar::{Builder, Header};
use thiserror::Error;

use crate::{
    graph::{HashedNodeWithEntries, NodeEntry},
    FsError, GraphError, NameStr, ObjectTree, WriteBytes,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("FsError: {0}")]
    Fs(#[from] FsError),
    #[error("GraphError: {0}")]
    Graph(#[from] GraphError),
    #[error("IoError: {0}")]
    Io(#[from] std::io::Error),
    #[error("TryFromIntError: {0}")]
    TryFromInt(#[from] TryFromIntError),
}

pub struct MemoryWriter {
    bytes: Vec<u8>,
}

impl MemoryWriter {
    pub async fn new<T>(tree: &ObjectTree<T>) -> Result<Self, MemoryError>
    where
        T: Clone + NameStr + WriteBytes + Send + Sync + 'static,
    {
        let (graph, root_idx) = tree.as_petgraph();
        let mut tar_builder = Builder::new(Vec::new());

        let mut dfspo = DfsPostOrder::new(graph, root_idx);

        while let Some(node_idx) = dfspo.next(graph) {
            let node = graph[node_idx].clone();

            let mut entries = Vec::new();
            for child_idx in graph.neighbors_directed(node_idx, Outgoing) {
                let child_node = &graph[child_idx];
                entries.push(NodeEntry::new(
                    child_node.kind(),
                    child_node.hash(),
                    child_node.name(),
                ));
            }

            let tar_entry = HashedNodeWithEntries::new(node, entries);
            write_tar_entry(
                &mut tar_builder,
                Path::new(".")
                    .join("objects")
                    .join(tar_entry.hash().to_string()),
                &tar_entry.to_bytes()?,
            )
            .await?;
        }

        write_tar_entry(
            &mut tar_builder,
            Path::new(".").join("refs").join("root"),
            graph[root_idx].hash().to_string().as_bytes(),
        )
        .await?;
        tar_builder.finish()?;

        Ok(MemoryWriter {
            bytes: tar_builder.into_inner()?,
        })
    }

    pub fn bytes(self) -> Vec<u8> {
        self.bytes
    }
}

async fn write_tar_entry(
    tar_builder: &mut Builder<Vec<u8>>,
    path: PathBuf,
    entry: &[u8],
) -> Result<(), MemoryError> {
    let mut tar_entry_header = Header::new_gnu();
    tar_entry_header.set_path(&path)?;
    tar_entry_header.set_size(entry.len().try_into()?);
    tar_entry_header.set_cksum();

    tar_builder.append(&dbg!(tar_entry_header), entry)?;

    Ok(())
}
