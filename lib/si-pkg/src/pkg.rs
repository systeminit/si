use core::fmt;
use std::{
    collections::HashMap,
    convert::Infallible,
    path::Path,
    sync::Arc,
};

use chrono::{DateTime, Utc};
use object_tree::{
    GraphError, Hash, HashedNode, NameStr, NodeChild, ObjectTree, TarReadError, TarWriter,
    TarWriterError,
};
use petgraph::prelude::*;
use thiserror::Error;

mod action_func;
mod attr_func_input;
mod func;
mod func_description;
mod leaf_function;
mod map_key_func;
mod prop;
mod schema;
mod si_prop_func;
mod socket;
mod validation;
mod variant;

pub use {
    action_func::*, attr_func_input::*, func::*, func_description::*, leaf_function::*,
    map_key_func::*, prop::*, schema::*, si_prop_func::*, socket::*, validation::*, variant::*,
};

use crate::{
    node::{CategoryNode, PkgNode},
    spec::{FuncSpec, PkgSpec, SchemaVariantSpecPropRoot, SpecError},
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum SiPkgError {
    #[error("Package missing required category: {0}")]
    CategoryNotFound(&'static str),
    #[error(transparent)]
    Graph(#[from] GraphError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Memory(#[from] TarWriterError),
    #[error("node not found with hash={0}")]
    NodeWithHashNotFound(Hash),
    #[error("node not found with name={0}")]
    NodeWithNameNotFound(String),
    #[error("found multiple pkg node domain props for variant with hash={0}")]
    PropRootMultipleFound(SchemaVariantSpecPropRoot, Hash),
    #[error("could not find pkg node root prop {0} for variant with hash={1}")]
    PropRootNotFound(SchemaVariantSpecPropRoot, Hash),
    #[error("SiPkg prop tree is invalid: {0}")]
    PropTreeInvalid(String),
    #[error("Schema Variant missing required child: {0}")]
    SchemaVariantChildNotFound(&'static str),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Spec(#[from] SpecError),
    #[error(transparent)]
    TarRead(#[from] TarReadError),
    #[error("unexpected pkg node type; expected={0}, actual={1}")]
    UnexpectedPkgNodeType(&'static str, &'static str),
    #[error("Validation spec missing required field: {0}")]
    ValidationMissingField(String),
    #[error("error while visiting prop: {0}")]
    VisitProp(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl SiPkgError {
    fn prop_tree_invalid(message: impl Into<String>) -> Self {
        Self::PropTreeInvalid(message.into())
    }
}

pub type PkgResult<T> = Result<T, SiPkgError>;

impl SiPkgError {
    pub fn visit_prop(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::VisitProp(Box::new(source))
    }
}

impl From<Infallible> for SiPkgError {
    fn from(_value: Infallible) -> Self {
        unreachable!("infallible will not error")
    }
}

#[derive(Clone, Debug)]
pub struct SiPkg {
    tree: Arc<ObjectTree<PkgNode>>,
}

impl SiPkg {
    pub async fn load_from_file(path: impl AsRef<Path>) -> PkgResult<Self> {
        let file_data = tokio::fs::read(&path).await?;
        Self::load_from_bytes(file_data)
    }

    pub fn load_from_bytes(bytes: Vec<u8>) -> PkgResult<Self> {
        let tree: ObjectTree<PkgNode> = ObjectTree::<PkgNode>::read_from_tar(bytes)?;

        Ok(Self {
            tree: Arc::new(tree),
        })
    }

    pub fn load_from_spec<I>(spec: I) -> PkgResult<Self>
    where
        I: TryInto<PkgSpec>,
        I::Error: Into<SiPkgError>,
    {
        let spec = spec.try_into().map_err(Into::into)?;
        let tree = ObjectTree::create_from_root(spec.as_node_with_children())?;

        Ok(Self {
            tree: Arc::new(tree),
        })
    }

    pub fn write_to_bytes(&self) -> PkgResult<Vec<u8>> {
        Ok(TarWriter::new(&self.tree)?.bytes())
    }

    pub fn metadata(&self) -> PkgResult<SiPkgMetadata> {
        let (graph, root_idx) = self.as_petgraph();

        SiPkgMetadata::from_graph(graph, root_idx)
    }

    pub fn hash(&self) -> PkgResult<Hash> {
        Ok(self.metadata()?.hash())
    }

    pub fn funcs_by_unique_id(&self) -> PkgResult<HashMap<Hash, SiPkgFunc>> {
        let func_map: HashMap<Hash, SiPkgFunc> = self
            .funcs()?
            .drain(..)
            .map(|func| (func.unique_id(), func))
            .collect();

        Ok(func_map)
    }

    pub fn funcs(&self) -> PkgResult<Vec<SiPkgFunc>> {
        let (graph, root_idx) = self.as_petgraph();

        let node_idxs = func_node_idxs(graph, root_idx)?;
        let mut funcs = Vec::with_capacity(node_idxs.len());
        for node_idx in node_idxs {
            funcs.push(SiPkgFunc::from_graph(graph, node_idx)?);
        }

        Ok(funcs)
    }

    pub fn schemas(&self) -> PkgResult<Vec<SiPkgSchema>> {
        let (graph, root_idx) = self.as_petgraph();

        let node_idxs = schema_node_idxs(graph, root_idx)?;
        let mut schemas = Vec::with_capacity(node_idxs.len());

        for node_idx in node_idxs {
            schemas.push(SiPkgSchema::from_graph(graph, node_idx)?);
        }

        Ok(schemas)
    }

    pub fn schema_by_name(&self, name: impl AsRef<str>) -> PkgResult<SiPkgSchema> {
        let (graph, root_idx) = self.as_petgraph();

        let node_idx = idx_for_name(graph, schema_node_idxs(graph, root_idx)?.into_iter(), name)?;

        SiPkgSchema::from_graph(graph, node_idx)
    }

    pub fn schema_by_hash(&self, hash: Hash) -> PkgResult<SiPkgSchema> {
        let (graph, root_idx) = self.as_petgraph();

        let node_idx = idx_for_hash(graph, schema_node_idxs(graph, root_idx)?.into_iter(), hash)?;

        SiPkgSchema::from_graph(graph, node_idx)
    }

    pub fn as_petgraph(&self) -> (&Graph<HashedNode<PkgNode>, ()>, NodeIndex) {
        self.tree.as_petgraph()
    }

    pub async fn to_spec(&self) -> PkgResult<PkgSpec> {
        let mut builder = PkgSpec::builder();

        let metadata = self.metadata()?;

        builder
            .name(metadata.name())
            .description(metadata.description())
            .version(metadata.version())
            .created_at(metadata.created_at())
            .created_by(metadata.created_by());

        for func in self.funcs()? {
            builder.func(FuncSpec::try_from(func)?);
        }

        for schema in self.schemas()? {
            builder.schema(schema.to_spec().await?);
        }

        Ok(builder.build()?)
    }
}

fn idx_for_name(
    graph: &Graph<HashedNode<PkgNode>, ()>,
    mut idx_iter: impl Iterator<Item = NodeIndex>,
    name: impl AsRef<str>,
) -> PkgResult<NodeIndex> {
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
) -> PkgResult<NodeIndex> {
    let node_idx = idx_iter
        .find(|node_idx| graph[*node_idx].hash() == hash)
        .ok_or_else(|| SiPkgError::NodeWithHashNotFound(hash))?;

    Ok(node_idx)
}

fn category_node_idxs(
    category_node: CategoryNode,
    graph: &Graph<HashedNode<PkgNode>, ()>,
    root_idx: NodeIndex,
) -> PkgResult<Vec<NodeIndex>> {
    let node_idxs = graph
        .neighbors_directed(root_idx, Outgoing)
        .find(|node_idx| match &graph[*node_idx].inner() {
            PkgNode::Category(node) => *node == category_node,
            _ => false,
        })
        .ok_or(SiPkgError::CategoryNotFound(category_node.kind_str()))?;

    Ok(graph.neighbors_directed(node_idxs, Outgoing).collect())
}

fn schema_node_idxs(
    graph: &Graph<HashedNode<PkgNode>, ()>,
    root_idx: NodeIndex,
) -> PkgResult<Vec<NodeIndex>> {
    category_node_idxs(CategoryNode::Schemas, graph, root_idx)
}

fn func_node_idxs(
    graph: &Graph<HashedNode<PkgNode>, ()>,
    root_idx: NodeIndex,
) -> PkgResult<Vec<NodeIndex>> {
    category_node_idxs(CategoryNode::Funcs, graph, root_idx)
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
    fn from_graph(graph: &Graph<HashedNode<PkgNode>, ()>, node_idx: NodeIndex) -> PkgResult<Self> {
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
