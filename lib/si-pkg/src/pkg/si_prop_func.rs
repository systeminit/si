use object_tree::{Hash, HashedNode};
use petgraph::prelude::*;

use super::{PkgResult, SiPkgError, Source};

use crate::{
    node::PkgNode, AttrFuncInputSpec, FuncUniqueId, SiPkgAttrFuncInput, SiPropFuncSpec,
    SiPropFuncSpecKind,
};

#[derive(Clone, Debug)]
pub struct SiPkgSiPropFunc<'a> {
    kind: SiPropFuncSpecKind,
    func_unique_id: FuncUniqueId,
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
                ))
            }
        };

        Ok(Self {
            kind: node.kind,
            func_unique_id: node.func_unique_id,
            hash: hashed_node.hash(),
            source: Source::new(graph, node_idx),
        })
    }

    pub fn kind(&self) -> SiPropFuncSpecKind {
        self.kind
    }

    pub fn func_unique_id(&self) -> FuncUniqueId {
        self.func_unique_id
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
            .build()?)
    }
}
