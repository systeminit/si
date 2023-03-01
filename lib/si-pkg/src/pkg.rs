use core::fmt;
use std::{
    future::Future,
    path::{Path, PathBuf},
    sync::Arc,
};

use chrono::{DateTime, Utc};
use object_tree::{
    FsError, GraphError, Hash, HashedNode, NameStr, NodeChild, ObjectTree, TreeFileSystemReader,
    TreeFileSystemWriter,
};
use petgraph::prelude::*;
use thiserror::Error;
use url::Url;

use crate::{
    node::{CategoryNode, PkgNode, PropNode},
    spec::Package,
};

#[derive(Debug, Error)]
pub enum SiPkgError {
    #[error("could not find pkg node category {0}")]
    CategoryNotFound(&'static str),
    #[error("could not find pkg node domain prop for variant with hash={0}")]
    DomainPropNotFound(Hash),
    #[error("found multiple pkg node domain props for variant with hash={0}")]
    DomainPropMultipleFound(Hash),
    #[error(transparent)]
    Fs(#[from] FsError),
    #[error(transparent)]
    Graph(#[from] GraphError),
    #[error("node not found with hash={0}")]
    NodeWithHashNotFound(Hash),
    #[error("node not found with name={0}")]
    NodeWithNameNotFound(String),
    #[error("unexpected pkg node type; expected={0}, actual={1}")]
    UnexpectedPkgNodeType(&'static str, &'static str),
    #[error("error while visiting prop: {0}")]
    VisitProp(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl SiPkgError {
    pub fn visit_prop(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::VisitProp(Box::new(source))
    }
}

#[derive(Clone, Debug)]
pub struct SiPkg {
    tree: Arc<ObjectTree<PkgNode>>,
}

impl SiPkg {
    pub async fn load_from_file(path: impl Into<PathBuf>) -> Result<Self, SiPkgError> {
        let tree: ObjectTree<PkgNode> = TreeFileSystemReader::tar(path).await?.read().await?;

        Ok(Self {
            tree: Arc::new(tree),
        })
    }

    pub async fn load_from_dir(path: impl Into<PathBuf>) -> Result<Self, SiPkgError> {
        let tree: ObjectTree<PkgNode> = TreeFileSystemReader::physical(path).read().await?;

        Ok(Self {
            tree: Arc::new(tree),
        })
    }

    pub fn load_from_spec(spec: Package) -> Result<Self, SiPkgError> {
        let tree = ObjectTree::create_from_root(spec.as_node_with_children())?;

        Ok(Self {
            tree: Arc::new(tree),
        })
    }

    pub async fn write_to_file(&self, path: impl Into<PathBuf>) -> Result<(), SiPkgError> {
        TreeFileSystemWriter::tar(path)
            .await?
            .write(&self.tree)
            .await
            .map_err(Into::into)
    }

    pub async fn write_to_dir(&self, path: impl AsRef<Path>) -> Result<(), SiPkgError> {
        TreeFileSystemWriter::physical(path)
            .write(&self.tree)
            .await
            .map_err(Into::into)
    }

    pub fn metadata(&self) -> Result<SiPkgMetadata, SiPkgError> {
        let (graph, root_idx) = self.as_petgraph();

        SiPkgMetadata::from_graph(graph, root_idx)
    }

    pub fn hash(&self) -> Result<Hash, SiPkgError> {
        Ok(self.metadata()?.hash())
    }

    pub fn schemas(&self) -> Result<Vec<SiPkgSchema>, SiPkgError> {
        let (graph, root_idx) = self.as_petgraph();

        let node_idxs = schema_node_idxs(graph, root_idx)?;
        let mut schemas = Vec::with_capacity(node_idxs.len());

        for node_idx in node_idxs {
            schemas.push(SiPkgSchema::from_graph(graph, node_idx)?);
        }

        Ok(schemas)
    }

    pub fn schema_by_name(&self, name: impl AsRef<str>) -> Result<SiPkgSchema, SiPkgError> {
        let (graph, root_idx) = self.as_petgraph();

        let node_idx = idx_for_name(graph, schema_node_idxs(graph, root_idx)?.into_iter(), name)?;

        SiPkgSchema::from_graph(graph, node_idx)
    }

    pub fn schema_by_hash(&self, hash: Hash) -> Result<SiPkgSchema, SiPkgError> {
        let (graph, root_idx) = self.as_petgraph();

        let node_idx = idx_for_hash(graph, schema_node_idxs(graph, root_idx)?.into_iter(), hash)?;

        SiPkgSchema::from_graph(graph, node_idx)
    }

    pub fn as_petgraph(&self) -> (&Graph<HashedNode<PkgNode>, ()>, NodeIndex) {
        self.tree.as_petgraph()
    }
}

fn idx_for_name(
    graph: &Graph<HashedNode<PkgNode>, ()>,
    mut idx_iter: impl Iterator<Item = NodeIndex>,
    name: impl AsRef<str>,
) -> Result<NodeIndex, SiPkgError> {
    let name = name.as_ref();
    let node_idx = idx_iter
        .find(|node_idx| graph[*node_idx].name() == name)
        .ok_or_else(|| SiPkgError::NodeWithNameNotFound(name.to_string()))?;

    Ok(node_idx)
}

fn idx_for_hash(
    graph: &Graph<HashedNode<PkgNode>, ()>,
    mut idx_iter: impl Iterator<Item = NodeIndex>,
    hash: Hash,
) -> Result<NodeIndex, SiPkgError> {
    let node_idx = idx_iter
        .find(|node_idx| graph[*node_idx].hash() == hash)
        .ok_or_else(|| SiPkgError::NodeWithHashNotFound(hash))?;

    Ok(node_idx)
}

fn schema_node_idxs(
    graph: &Graph<HashedNode<PkgNode>, ()>,
    root_idx: NodeIndex,
) -> Result<Vec<NodeIndex>, SiPkgError> {
    let schemas_idx = graph
        .neighbors_directed(root_idx, Outgoing)
        .find(|node_idx| match &graph[*node_idx].inner() {
            PkgNode::Category(node) => match node {
                CategoryNode::Schemas => true,
            },
            _ => false,
        })
        .ok_or(SiPkgError::CategoryNotFound(
            CategoryNode::Schemas.kind_str(),
        ))?;

    Ok(graph.neighbors_directed(schemas_idx, Outgoing).collect())
}

#[derive(Clone)]
pub struct Source<'a> {
    graph: &'a Graph<HashedNode<PkgNode>, ()>,
    node_idx: NodeIndex,
}

impl<'a> Source<'a> {
    fn new(graph: &'a Graph<HashedNode<PkgNode>, ()>, node_idx: NodeIndex) -> Self {
        Self { graph, node_idx }
    }
}

impl<'a> fmt::Debug for Source<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Source")
            .field("graph", &"...")
            .field("node_idx", &self.node_idx)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct SiPkgMetadata {
    name: String,
    version: String,
    description: String,
    created_at: DateTime<Utc>,
    created_by: String,

    hash: Hash,
}

impl SiPkgMetadata {
    fn from_graph(
        graph: &Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> Result<Self, SiPkgError> {
        let metadata_hashed_node = &graph[node_idx];
        let metadata_node = match metadata_hashed_node.inner() {
            PkgNode::Package(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::PACKAGE_KIND_STR,
                    unexpected.node_kind_str(),
                ))
            }
        };

        Ok(Self {
            name: metadata_node.name,
            version: metadata_node.version,
            description: metadata_node.description,
            created_at: metadata_node.created_at,
            created_by: metadata_node.created_by,
            hash: metadata_hashed_node.hash(),
        })
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn version(&self) -> &str {
        self.version.as_ref()
    }

    pub fn description(&self) -> &str {
        self.description.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn created_by(&self) -> &str {
        self.created_by.as_ref()
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }
}

#[derive(Clone, Debug)]
pub struct SiPkgSchema<'a> {
    name: String,
    category: String,

    hash: Hash,

    source: Source<'a>,
}

impl<'a> SiPkgSchema<'a> {
    fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> Result<Self, SiPkgError> {
        let schema_hashed_node = &graph[node_idx];
        let schema_node = match schema_hashed_node.inner() {
            PkgNode::Schema(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::SCHEMA_KIND_STR,
                    unexpected.node_kind_str(),
                ))
            }
        };

        let schema = Self {
            name: schema_node.name,
            category: schema_node.category,
            hash: schema_hashed_node.hash(),
            source: Source::new(graph, node_idx),
        };

        Ok(schema)
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn category(&self) -> &str {
        self.category.as_ref()
    }

    pub fn variants(&self) -> Result<Vec<SiPkgSchemaVariant<'a>>, SiPkgError> {
        let mut variants = vec![];
        for schema_variant_idx in self
            .source
            .graph
            .neighbors_directed(self.source.node_idx, Outgoing)
        {
            variants.push(SiPkgSchemaVariant::from_graph(
                self.source.graph,
                schema_variant_idx,
            )?);
        }

        Ok(variants)
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }
}

#[derive(Clone, Debug)]
pub struct SiPkgSchemaVariant<'a> {
    name: String,
    link: Option<Url>,
    color: Option<String>,

    hash: Hash,

    source: Source<'a>,
}

impl<'a> SiPkgSchemaVariant<'a> {
    fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> Result<Self, SiPkgError> {
        let schema_variant_hashed_node = &graph[node_idx];
        let schema_variant_node = match schema_variant_hashed_node.inner() {
            PkgNode::SchemaVariant(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::SCHEMA_VARIANT_KIND_STR,
                    unexpected.node_kind_str(),
                ))
            }
        };

        let schema_variant = Self {
            name: schema_variant_node.name,
            link: schema_variant_node.link,
            color: schema_variant_node.color,
            hash: schema_variant_hashed_node.hash(),
            source: Source::new(graph, node_idx),
        };

        Ok(schema_variant)
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn link(&self) -> Option<&Url> {
        self.link.as_ref()
    }

    pub fn color(&self) -> Option<&str> {
        self.color.as_deref()
    }

    pub async fn visit_prop_tree<F, Fut, I, C>(
        &'a self,
        process_prop_fn: F,
        parent_id: Option<I>,
        context: &'a C,
    ) -> Result<(), SiPkgError>
    where
        F: Fn(SiPkgProp<'a>, Option<I>, &'a C) -> Fut,
        Fut: Future<Output = Result<Option<I>, SiPkgError>>,
        I: Copy,
    {
        let mut child_node_idxs: Vec<_> = self
            .source
            .graph
            .neighbors_directed(self.source.node_idx, Outgoing)
            .collect();
        let domain_node_idx = match child_node_idxs.pop() {
            Some(idx) => idx,
            None => return Err(SiPkgError::DomainPropNotFound(self.hash())),
        };
        if !child_node_idxs.is_empty() {
            return Err(SiPkgError::DomainPropMultipleFound(self.hash()));
        }

        let mut stack: Vec<(_, Option<I>)> = Vec::new();
        // Skip processing the domain prop as a `dal::SchemaVariant` already guarantees such a prop
        // has already been created. Rather, we will push all immediate children of the domain prop
        // to be ready for processing.
        for child_idx in self
            .source
            .graph
            .neighbors_directed(domain_node_idx, Outgoing)
        {
            stack.push((
                SiPkgProp::from_graph(self.source.graph, child_idx)?,
                parent_id,
            ));
        }

        while let Some((prop, parent_id)) = stack.pop() {
            let node_idx = prop.source().node_idx;
            let new_id = process_prop_fn(prop, parent_id, context).await?;

            for child_idx in self.source.graph.neighbors_directed(node_idx, Outgoing) {
                stack.push((SiPkgProp::from_graph(self.source.graph, child_idx)?, new_id));
            }
        }

        Ok(())
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }
}

#[derive(Clone, Debug)]
pub enum SiPkgProp<'a> {
    String {
        name: String,
        hash: Hash,
        source: Source<'a>,
    },
    Number {
        name: String,
        hash: Hash,
        source: Source<'a>,
    },
    Boolean {
        name: String,
        hash: Hash,
        source: Source<'a>,
    },
    Map {
        name: String,
        // type_prop: Box<Prop>,
        hash: Hash,
        source: Source<'a>,
    },
    Array {
        name: String,
        // type_prop: Box<Prop>,
        hash: Hash,
        source: Source<'a>,
    },
    Object {
        name: String,
        // entries: Vec<Prop>,
        hash: Hash,
        source: Source<'a>,
    },
}

impl<'a> SiPkgProp<'a> {
    fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> Result<Self, SiPkgError> {
        let prop_hashed_node = &graph[node_idx];
        let prop_node = match prop_hashed_node.inner() {
            PkgNode::Prop(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::PROP_KIND_STR,
                    unexpected.node_kind_str(),
                ))
            }
        };

        let hash = prop_hashed_node.hash();
        let source = Source::new(graph, node_idx);

        Ok(match prop_node {
            PropNode::String { name } => Self::String { name, hash, source },
            PropNode::Integer { name } => Self::Number { name, hash, source },
            PropNode::Boolean { name } => Self::Boolean { name, hash, source },
            PropNode::Map { name } => Self::Map { name, hash, source },
            PropNode::Array { name } => Self::Array { name, hash, source },
            PropNode::Object { name } => Self::Object { name, hash, source },
        })
    }

    pub fn name(&self) -> &str {
        match self {
            Self::String { name, .. }
            | Self::Number { name, .. }
            | Self::Boolean { name, .. }
            | Self::Map { name, .. }
            | Self::Array { name, .. }
            | Self::Object { name, .. } => name,
        }
    }

    pub fn hash(&self) -> Hash {
        match self {
            Self::String { hash, .. }
            | Self::Number { hash, .. }
            | Self::Boolean { hash, .. }
            | Self::Map { hash, .. }
            | Self::Array { hash, .. }
            | Self::Object { hash, .. } => *hash,
        }
    }

    fn source(&self) -> &Source<'a> {
        match self {
            Self::String { source, .. }
            | Self::Number { source, .. }
            | Self::Boolean { source, .. }
            | Self::Map { source, .. }
            | Self::Array { source, .. }
            | Self::Object { source, .. } => source,
        }
    }
}
