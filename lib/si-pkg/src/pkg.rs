use core::fmt;
use std::{
    collections::HashMap,
    convert::Infallible,
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
    node::{CategoryNode, PkgNode, PropNode, SchemaVariantChildNode},
    spec::{FuncSpecBackendKind, FuncSpecBackendResponseType, PkgSpec, SpecError},
};

#[derive(Debug, Error)]
pub enum SiPkgError {
    #[error("Package missing required category: {0}")]
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
    #[error(transparent)]
    Spec(#[from] SpecError),
    #[error("unexpected pkg node type; expected={0}, actual={1}")]
    UnexpectedPkgNodeType(&'static str, &'static str),
    #[error("error while visiting prop: {0}")]
    VisitProp(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("Schema Variant missing required child: {0}")]
    SchemaVariantChildNotFound(&'static str),
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
    pub async fn load_from_file(path: impl Into<PathBuf>) -> PkgResult<Self> {
        let tree: ObjectTree<PkgNode> = TreeFileSystemReader::tar(path).await?.read().await?;

        Ok(Self {
            tree: Arc::new(tree),
        })
    }

    pub async fn load_from_dir(path: impl Into<PathBuf>) -> PkgResult<Self> {
        let tree: ObjectTree<PkgNode> = TreeFileSystemReader::physical(path).read().await?;

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

    pub async fn write_to_file(&self, path: impl Into<PathBuf>) -> PkgResult<()> {
        TreeFileSystemWriter::tar(path)
            .await?
            .write(&self.tree)
            .await
            .map_err(Into::into)
    }

    pub async fn write_to_dir(&self, path: impl AsRef<Path>) -> PkgResult<()> {
        TreeFileSystemWriter::physical(path)
            .write(&self.tree)
            .await
            .map_err(Into::into)
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

#[derive(Clone, Debug)]
pub struct SiPkgFunc<'a> {
    name: String,
    display_name: Option<String>,
    description: Option<String>,
    handler: String,
    code_base64: String,
    backend_kind: FuncSpecBackendKind,
    response_type: FuncSpecBackendResponseType,
    hidden: bool,
    link: Option<Url>,
    unique_id: Hash,

    hash: Hash,
    source: Source<'a>,
}

impl<'a> SiPkgFunc<'a> {
    fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
        let func_hashed_node = &graph[node_idx];
        let func_node = match func_hashed_node.inner() {
            PkgNode::Func(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::FUNC_KIND_STR,
                    unexpected.node_kind_str(),
                ))
            }
        };

        Ok(Self {
            name: func_node.name,
            display_name: func_node.display_name,
            description: func_node.description,
            handler: func_node.handler,
            code_base64: func_node.code_base64,
            backend_kind: func_node.backend_kind,
            response_type: func_node.response_type,
            hidden: func_node.hidden,
            link: func_node.link,
            hash: func_hashed_node.hash(),
            unique_id: func_node.unique_id,
            source: Source::new(graph, node_idx),
        })
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn handler(&self) -> &str {
        self.handler.as_ref()
    }

    pub fn code_base64(&self) -> &str {
        self.code_base64.as_ref()
    }

    pub fn backend_kind(&self) -> FuncSpecBackendKind {
        self.backend_kind
    }

    pub fn response_type(&self) -> FuncSpecBackendResponseType {
        self.response_type
    }

    pub fn hidden(&self) -> bool {
        self.hidden
    }

    pub fn link(&self) -> Option<&Url> {
        self.link.as_ref()
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }

    pub fn unique_id(&self) -> Hash {
        self.unique_id
    }

    pub fn source(&self) -> &Source<'a> {
        &self.source
    }
}

#[derive(Clone, Debug)]
pub struct SiPkgSchema<'a> {
    name: String,
    category: String,
    category_name: Option<String>,

    hash: Hash,

    source: Source<'a>,
}

impl<'a> SiPkgSchema<'a> {
    fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
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
            category_name: schema_node.category_name,
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

    pub fn category_name(&self) -> Option<&str> {
        self.category_name.as_deref()
    }

    pub fn variants(&self) -> PkgResult<Vec<SiPkgSchemaVariant<'a>>> {
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
    ) -> PkgResult<Self> {
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

    pub async fn qualifications(&self) -> PkgResult<Vec<SiPkgQualification>> {
        let qual_child_idxs = self
            .source
            .graph
            .neighbors_directed(self.source.node_idx, Outgoing)
            .find(|node_idx| {
                matches!(
                    &self.source.graph[*node_idx].inner(),
                    PkgNode::SchemaVariantChild(SchemaVariantChildNode::Qualifications)
                )
            })
            .ok_or(SiPkgError::CategoryNotFound(
                SchemaVariantChildNode::Qualifications.kind_str(),
            ))?;

        let child_node_idxs: Vec<_> = self
            .source
            .graph
            .neighbors_directed(qual_child_idxs, Outgoing)
            .collect();

        let mut qualifications = vec![];
        for child_idx in child_node_idxs {
            qualifications.push(SiPkgQualification::from_graph(
                self.source.graph,
                child_idx,
            )?);
        }

        Ok(qualifications)
    }

    pub async fn visit_prop_tree<F, Fut, I, C>(
        &'a self,
        process_prop_fn: F,
        parent_id: Option<I>,
        context: &'a C,
    ) -> PkgResult<()>
    where
        F: Fn(SiPkgProp<'a>, Option<I>, &'a C) -> Fut,
        Fut: Future<Output = PkgResult<Option<I>>>,
        I: Copy,
    {
        let domain_idxs = self
            .source
            .graph
            .neighbors_directed(self.source.node_idx, Outgoing)
            .find(|node_idx| {
                matches!(
                    &self.source.graph[*node_idx].inner(),
                    PkgNode::SchemaVariantChild(SchemaVariantChildNode::Domain)
                )
            })
            .ok_or(SiPkgError::CategoryNotFound(
                SchemaVariantChildNode::Domain.kind_str(),
            ))?;

        let mut child_node_idxs: Vec<_> = self
            .source
            .graph
            .neighbors_directed(domain_idxs, Outgoing)
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
pub struct SiPkgQualification<'a> {
    func_unique_id: Hash,
    hash: Hash,
    source: Source<'a>,
}

impl<'a> SiPkgQualification<'a> {
    fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
        let qual_hashed_node = &graph[node_idx];
        let qual_node = match qual_hashed_node.inner() {
            PkgNode::Qualification(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::QUALIFICATION_KIND_STR,
                    unexpected.node_kind_str(),
                ))
            }
        };

        Ok(Self {
            func_unique_id: qual_node.func_unique_id,
            hash: qual_hashed_node.hash(),
            source: Source::new(graph, node_idx),
        })
    }

    pub fn func_unique_id(&self) -> Hash {
        self.func_unique_id
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }

    pub fn source(&self) -> &Source<'a> {
        &self.source
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
    ) -> PkgResult<Self> {
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
