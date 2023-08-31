use object_tree::{Hash, HashedNode};
use petgraph::prelude::*;

use super::{PkgResult, SiPkgError, Source};

use crate::{node::PkgNode, AttrFuncInputSpec, MapKeyFuncSpec, SiPkgAttrFuncInput};

#[derive(Clone, Debug)]
pub struct SiPkgMapKeyFunc<'a> {
    key: String,
    func_unique_id: String,
    hash: Hash,
    source: Source<'a>,
}

impl<'a> SiPkgMapKeyFunc<'a> {
    pub fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
        let hashed_node = &graph[node_idx];
        let node = match hashed_node.inner() {
            PkgNode::MapKeyFunc(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::MAP_KEY_FUNC_KIND_STR,
                    unexpected.node_kind_str(),
                ))
            }
        };

        Ok(Self {
            key: node.key.to_owned(),
            func_unique_id: node.func_unique_id.to_owned(),
            hash: hashed_node.hash(),
            source: Source::new(graph, node_idx),
        })
    }

    pub fn func_unique_id(&self) -> &str {
        self.func_unique_id.as_str()
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

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }

    pub fn source(&self) -> &Source<'a> {
        &self.source
    }
}

impl<'a> TryFrom<SiPkgMapKeyFunc<'a>> for MapKeyFuncSpec {
    type Error = SiPkgError;

    fn try_from(value: SiPkgMapKeyFunc<'a>) -> Result<Self, Self::Error> {
        let mut builder = MapKeyFuncSpec::builder();
        for input in value.inputs()? {
            builder.input(AttrFuncInputSpec::try_from(input)?);
        }

        Ok(builder
            .key(value.key)
            .func_unique_id(value.func_unique_id)
            .build()?)
    }
}
