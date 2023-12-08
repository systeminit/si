use object_tree::{Hash, HashedNode};
use petgraph::prelude::*;

use super::{PkgResult, SiPkgAttrFuncInput, SiPkgError, Source};

use crate::spec::SocketSpecData;
use crate::{node::PkgNode, SocketSpec, SocketSpecArity, SocketSpecKind};

#[derive(Clone, Debug)]
pub struct SiPkgSocketData {
    name: String,
    connection_annotations: String,
    func_unique_id: Option<String>,
    kind: SocketSpecKind,
    arity: SocketSpecArity,
    ui_hidden: bool,
}

impl SiPkgSocketData {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn connection_annotations(&self) -> &str {
        self.connection_annotations.as_str()
    }
    pub fn func_unique_id(&self) -> Option<&str> {
        self.func_unique_id.as_deref()
    }

    pub fn kind(&self) -> SocketSpecKind {
        self.kind
    }

    pub fn arity(&self) -> SocketSpecArity {
        self.arity
    }

    pub fn ui_hidden(&self) -> bool {
        self.ui_hidden
    }
}

#[derive(Clone, Debug)]
pub struct SiPkgSocket<'a> {
    name: String,
    data: Option<SiPkgSocketData>,
    unique_id: Option<String>,

    hash: Hash,
    source: Source<'a>,
}

impl<'a> SiPkgSocket<'a> {
    pub fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
        let hashed_node = &graph[node_idx];
        let node = match hashed_node.inner() {
            PkgNode::Socket(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::SOCKET_KIND_STR,
                    unexpected.node_kind_str(),
                ))
            }
        };

        Ok(Self {
            name: node.name,
            data: node.data.map(|data| SiPkgSocketData {
                name: data.name,
                connection_annotations: data.connection_annotations,
                kind: data.kind,
                func_unique_id: data.func_unique_id,
                arity: data.arity,
                ui_hidden: data.ui_hidden,
            }),
            unique_id: node.unique_id,

            hash: hashed_node.hash(),
            source: Source::new(graph, node_idx),
        })
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }

    pub fn data(&self) -> Option<&SiPkgSocketData> {
        self.data.as_ref()
    }

    pub fn unique_id(&self) -> Option<&str> {
        self.unique_id.as_deref()
    }

    pub fn source(&self) -> &Source<'a> {
        &self.source
    }
}

impl<'a> TryFrom<SiPkgSocket<'a>> for SocketSpec {
    type Error = SiPkgError;

    fn try_from(value: SiPkgSocket<'a>) -> Result<Self, Self::Error> {
        let mut builder = SocketSpec::builder();

        builder
            .name(&value.name)
            .unique_id(value.unique_id.to_owned());

        if let Some(data) = &value.data {
            let mut data_builder = SocketSpecData::builder();
            if let Some(func_unique_id) = &data.func_unique_id {
                data_builder.func_unique_id(func_unique_id);
            }
            data_builder
                .name(&data.name)
                .connection_annotations(&data.connection_annotations)
                .kind(data.kind)
                .arity(data.arity)
                .ui_hidden(data.ui_hidden);
        }

        for input in value.inputs()? {
            builder.input(input.try_into()?);
        }

        Ok(builder.build()?)
    }
}
