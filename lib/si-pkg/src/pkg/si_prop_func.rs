use object_tree::{
    Hash,
    HashedNode,
};
use petgraph::prelude::*;

use super::{
    PkgResult,
    SiPkgError,
    Source,
};
use crate::{
    AttrFuncInputSpec,
    HasUniqueId,
    SiPkgAttrFuncInput,
    SiPropFuncSpec,
    SiPropFuncSpecKind,
    node::PkgNode,
};

#[derive(Clone, Debug)]
pub struct SiPkgSiPropFunc<'a> {
    kind: SiPropFuncSpecKind,
    func_unique_id: String,
    unique_id: Option<String>,
    deleted: bool,

    hash: Hash,
    source: Source<'a>,
}

impl<'a> SiPkgSiPropFunc<'a> {
    pub fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
        let hashed_node = &graph[node_idx];
        let node = match hashed_node.inner() {
            PkgNode::SiPropFunc(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::SI_PROP_FUNC_KIND_STR,
                    unexpected.node_kind_str(),
                ));
            }
        };

        Ok(Self {
            kind: node.kind,
            func_unique_id: node.func_unique_id,
            unique_id: node.unique_id,
            deleted: node.deleted,

            hash: hashed_node.hash(),
            source: Source::new(graph, node_idx),
        })
    }

    pub fn kind(&self) -> SiPropFuncSpecKind {
        self.kind
    }

    pub fn func_unique_id(&self) -> &str {
        self.func_unique_id.as_str()
    }

    pub fn unique_id(&self) -> Option<&str> {
        self.unique_id.as_deref()
    }

    pub fn deleted(&self) -> bool {
        self.deleted
    }

    pub fn inputs(&self) -> PkgResult<Vec<SiPkgAttrFuncInput>> {
        let mut inputs = vec![];

        for idx in self
            .source
            .graph
            .neighbors_directed(self.source.node_idx, Outgoing)
        {
            inputs.push(SiPkgAttrFuncInput::from_graph(self.source.graph, idx)?);
        }

        Ok(inputs)
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }

    pub fn source(&self) -> &Source<'a> {
        &self.source
    }
}

impl<'a> HasUniqueId for SiPkgSiPropFunc<'a> {
    fn unique_id(&self) -> Option<&str> {
        self.unique_id()
    }
}

impl<'a> TryFrom<SiPkgSiPropFunc<'a>> for SiPropFuncSpec {
    type Error = SiPkgError;

    fn try_from(value: SiPkgSiPropFunc<'a>) -> Result<Self, Self::Error> {
        let mut builder = SiPropFuncSpec::builder();
        for input in value.inputs()? {
            builder.input(AttrFuncInputSpec::try_from(input)?);
        }

        Ok(builder
            .kind(value.kind)
            .func_unique_id(value.func_unique_id)
            .unique_id(value.unique_id)
            .deleted(value.deleted)
            .build()?)
    }
}
