use object_tree::{Hash, HashedNode};
use petgraph::prelude::*;

use url::Url;

use crate::{
    node::PkgNode,
    spec::{FuncArgumentKind, FuncSpecBackendKind, FuncSpecBackendResponseType},
};

use super::{PkgResult, SiPkgError, Source};

#[derive(Clone, Debug)]
pub struct SiPkgFuncArgument<'a> {
    name: String,
    kind: FuncArgumentKind,
    element_kind: Option<FuncArgumentKind>,

    hash: Hash,
    source: Source<'a>,
}

impl<'a> SiPkgFuncArgument<'a> {
    fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
        let hashed_node = &graph[node_idx];
        let node = match hashed_node.inner() {
            PkgNode::FuncArgument(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::FUNC_ARGUMENT_KIND_STR,
                    unexpected.node_kind_str(),
                ))
            }
        };

        Ok(Self {
            name: node.name,
            kind: node.kind,
            element_kind: node.element_kind,

            hash: hashed_node.hash(),
            source: Source::new(graph, node_idx),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> FuncArgumentKind {
        self.kind
    }

    pub fn element_kind(&self) -> Option<&FuncArgumentKind> {
        self.element_kind.as_ref()
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }

    pub fn source(&self) -> &Source<'a> {
        &self.source
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
    pub fn from_graph(
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

    pub fn arguments(&self) -> PkgResult<Vec<SiPkgFuncArgument>> {
        let mut arguments = vec![];
        for idx in self
            .source
            .graph
            .neighbors_directed(self.source.node_idx, Outgoing)
        {
            arguments.push(SiPkgFuncArgument::from_graph(self.source.graph, idx)?);
        }

        Ok(arguments)
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
