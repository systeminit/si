use object_tree::{Hash, HashedNode};
use petgraph::prelude::*;

use super::{PkgResult, SiPkgError, Source};
use crate::node::PkgNode;
use crate::{EdgeSpec, EdgeSpecKind};

#[derive(Clone, Debug)]
pub struct SiPkgEdge<'a> {
    edge_kind: EdgeSpecKind,
    from_component_unique_id: String,
    from_socket_name: String,
    to_component_unique_id: String,
    to_socket_name: String,
    creation_user_pk: Option<String>,
    deletion_user_pk: Option<String>,
    deleted_implicitly: bool,

    unique_id: String,
    deleted: bool,

    hash: Hash,
    source: Source<'a>,
}

impl<'a> SiPkgEdge<'a> {
    pub fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
        let hashed_node = &graph[node_idx];
        let node = match hashed_node.inner() {
            PkgNode::Edge(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::NODE_KIND_EDGE_STR,
                    unexpected.node_kind_str(),
                ))
            }
        };

        Ok(Self {
            edge_kind: node.edge_kind,
            from_component_unique_id: node.from_component_unique_id,
            from_socket_name: node.from_socket_name,
            to_component_unique_id: node.to_component_unique_id,
            to_socket_name: node.to_socket_name,

            creation_user_pk: node.creation_user_pk,
            deletion_user_pk: node.deletion_user_pk,
            deleted_implicitly: node.deleted_implicitly,

            unique_id: node.unique_id,
            deleted: node.deleted,

            hash: hashed_node.hash(),
            source: Source::new(graph, node_idx),
        })
    }

    pub fn edge_kind(&self) -> EdgeSpecKind {
        self.edge_kind
    }

    pub fn from_component_unique_id(&self) -> &str {
        self.from_component_unique_id.as_str()
    }

    pub fn from_socket_name(&self) -> &str {
        self.from_socket_name.as_str()
    }

    pub fn to_component_unique_id(&self) -> &str {
        self.to_component_unique_id.as_str()
    }

    pub fn to_socket_name(&self) -> &str {
        self.to_socket_name.as_str()
    }

    pub fn creation_user_pk(&self) -> Option<&str> {
        self.creation_user_pk.as_deref()
    }

    pub fn deletion_user_pk(&self) -> Option<&str> {
        self.deletion_user_pk.as_deref()
    }

    pub fn deleted_implicitly(&self) -> bool {
        self.deleted_implicitly
    }

    pub fn unique_id(&self) -> &str {
        self.unique_id.as_str()
    }

    pub fn deleted(&self) -> bool {
        self.deleted
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }

    pub fn source(&self) -> &Source<'a> {
        &self.source
    }
}

impl<'a> TryFrom<SiPkgEdge<'a>> for EdgeSpec {
    type Error = SiPkgError;

    fn try_from(value: SiPkgEdge<'a>) -> Result<Self, Self::Error> {
        Ok(EdgeSpec::builder()
            .edge_kind(value.edge_kind())
            .from_component_unique_id(value.from_component_unique_id())
            .from_socket_name(value.from_socket_name())
            .to_component_unique_id(value.to_component_unique_id())
            .to_socket_name(value.to_socket_name())
            .creation_user_pk(value.creation_user_pk().map(Into::into))
            .deletion_user_pk(value.deletion_user_pk().map(Into::into))
            .deleted_implicitly(value.deleted_implicitly())
            .unique_id(value.unique_id())
            .deleted(value.deleted())
            .build()?)
    }
}
