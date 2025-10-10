use core::fmt;
use std::{
    collections::HashMap,
    convert::Infallible,
    path::Path,
    sync::Arc,
};

use chrono::{
    DateTime,
    Utc,
};
use object_tree::{
    GraphError,
    Hash,
    HashedNode,
    NameStr,
    NodeChild,
    ObjectTree,
    TarReadError,
    TarWriter,
    TarWriterError,
};
use petgraph::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};
use strum::{
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
};
use thiserror::Error;

mod action_func;
mod attr_func_input;
mod attribute_value;
mod auth_func;
mod change_set;
mod component;
mod edge;
mod func;
mod leaf_function;
mod management_func;
mod map_key_func;
mod position;
mod prop;
mod root_prop_func;
mod schema;
mod si_prop_func;
mod socket;
mod variant;

pub use action_func::*;
pub use attr_func_input::*;
pub use attribute_value::*;
pub use auth_func::*;
pub use change_set::*;
pub use component::*;
pub use edge::*;
pub use func::*;
pub use leaf_function::*;
pub use management_func::*;
pub use map_key_func::*;
pub use position::*;
pub use prop::*;
pub use root_prop_func::*;
pub use schema::*;
pub use si_prop_func::*;
pub use socket::*;
pub use variant::*;

use crate::{
    node::{
        CategoryNode,
        PkgNode,
    },
    spec::{
        FuncSpec,
        PkgSpec,
        SchemaVariantSpecPropRoot,
        SpecError,
    },
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum SiPkgError {
    #[error("component pkg node {0} missing position child")]
    ComponentMissingPosition(String),
    #[error("graph error: {0}")]
    Graph(#[from] GraphError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("memory error: {0}")]
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
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("spec error: {0}")]
    Spec(#[from] SpecError),
    #[error("tar read error: {0}")]
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
        Self::load_from_bytes(&file_data)
    }

    pub fn load_from_bytes(bytes: &[u8]) -> PkgResult<Self> {
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

    pub fn funcs_by_unique_id(&self) -> PkgResult<HashMap<String, SiPkgFunc>> {
        let func_map: HashMap<String, SiPkgFunc> = self
            .funcs()?
            .drain(..)
            .map(|func| (func.unique_id().to_string(), func))
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

    pub fn funcs_for_name(&self, name: &str) -> PkgResult<Vec<SiPkgFunc>> {
        Ok(self
            .funcs()?
            .drain(..)
            .filter(|func| func.name() == name)
            .collect())
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

    pub fn change_sets(&self) -> PkgResult<Vec<SiPkgChangeSet>> {
        let (graph, root_idx) = self.as_petgraph();

        let node_idxs = category_node_idxs(CategoryNode::ChangeSets, graph, root_idx)?;

        let mut change_sets = Vec::with_capacity(node_idxs.len());

        for node_idx in node_idxs {
            change_sets.push(SiPkgChangeSet::from_graph(graph, node_idx)?);
        }

        Ok(change_sets)
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
            .kind(metadata.kind())
            .name(metadata.name())
            .description(metadata.description())
            .version(metadata.version())
            .created_at(metadata.created_at())
            .created_by(metadata.created_by());

        if let Some(workspace_pk) = metadata.workspace_pk() {
            builder.workspace_pk(workspace_pk);
        }

        if let Some(workspace_name) = metadata.workspace_name() {
            builder.workspace_name(workspace_name);
        }

        for func in self.funcs()? {
            builder.func(FuncSpec::try_from(func)?);
        }

        for schema in self.schemas()? {
            builder.schema(schema.to_spec().await?);
        }

        if let SiPkgKind::WorkspaceBackup = metadata.kind() {
            if let Some(default_change_set) = metadata.default_change_set() {
                builder.default_change_set(default_change_set);
            }

            for change_set in self.change_sets()? {
                builder.change_set(change_set.to_spec().await?);
            }
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
        });

    Ok(node_idxs
        .map(|node_idx| graph.neighbors_directed(node_idx, Outgoing).collect())
        .unwrap_or(vec![]))
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

impl fmt::Debug for Source<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Source")
            .field("graph", &"...")
            .field("node_idx", &self.node_idx)
            .finish()
    }
}

#[remain::sorted]
#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
    Eq,
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
    Copy,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum SiPkgKind {
    Module,
    WorkspaceBackup,
}

#[derive(Clone, Debug)]
pub struct SiPkgMetadata {
    kind: SiPkgKind,
    name: String,
    version: String,
    description: String,
    created_at: DateTime<Utc>,
    created_by: String,
    default_change_set: Option<String>,
    workspace_pk: Option<String>,
    workspace_name: Option<String>,
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
                ));
            }
        };

        Ok(Self {
            kind: metadata_node.kind,
            name: metadata_node.name,
            version: metadata_node.version,
            description: metadata_node.description,
            created_at: metadata_node.created_at,
            created_by: metadata_node.created_by,
            default_change_set: metadata_node.default_change_set,
            workspace_pk: metadata_node.workspace_pk,
            workspace_name: metadata_node.workspace_name,
            hash: metadata_hashed_node.hash(),
        })
    }

    pub fn kind(&self) -> SiPkgKind {
        self.kind
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

    pub fn default_change_set(&self) -> Option<&str> {
        self.default_change_set.as_deref()
    }

    pub fn workspace_pk(&self) -> Option<&str> {
        self.workspace_pk.as_deref()
    }

    pub fn workspace_name(&self) -> Option<&str> {
        self.workspace_name.as_deref()
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }
}
