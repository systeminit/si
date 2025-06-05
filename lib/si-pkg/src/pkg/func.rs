use chrono::{
    DateTime,
    Utc,
};
use object_tree::{
    Hash,
    HashedNode,
};
use petgraph::prelude::*;
use url::Url;

use super::{
    PkgResult,
    SiPkgError,
    Source,
};
use crate::{
    node::PkgNode,
    spec::{
        FuncArgumentKind,
        FuncArgumentSpec,
        FuncSpec,
        FuncSpecBackendKind,
        FuncSpecBackendResponseType,
        FuncSpecData,
    },
};

#[derive(Clone, Debug)]
pub struct SiPkgFuncArgument<'a> {
    name: String,
    kind: FuncArgumentKind,
    element_kind: Option<FuncArgumentKind>,
    unique_id: Option<String>,
    deleted: bool,

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
                ));
            }
        };

        Ok(Self {
            name: node.name,
            kind: node.kind,
            element_kind: node.element_kind,
            unique_id: node.unique_id,
            deleted: node.deleted,

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

    pub fn unique_id(&self) -> Option<&str> {
        self.unique_id.as_deref()
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

impl<'a> TryFrom<SiPkgFuncArgument<'a>> for FuncArgumentSpec {
    type Error = SiPkgError;

    fn try_from(value: SiPkgFuncArgument<'a>) -> Result<Self, Self::Error> {
        Ok(FuncArgumentSpec::builder()
            .name(value.name)
            .kind(value.kind)
            .element_kind(value.element_kind)
            .unique_id(value.unique_id.to_owned())
            .deleted(value.deleted)
            .build()?)
    }
}

#[derive(Clone, Debug)]
pub struct SiPkgFuncData {
    name: String,
    display_name: Option<String>,
    description: Option<String>,
    handler: String,
    code_base64: String,
    backend_kind: FuncSpecBackendKind,
    response_type: FuncSpecBackendResponseType,
    is_transformation: bool,
    hidden: bool,
    link: Option<Url>,
    last_updated_at: Option<DateTime<Utc>>,
}

impl SiPkgFuncData {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn handler(&self) -> &str {
        self.handler.as_str()
    }

    pub fn code_base64(&self) -> &str {
        self.code_base64.as_str()
    }

    pub fn backend_kind(&self) -> FuncSpecBackendKind {
        self.backend_kind
    }

    pub fn response_type(&self) -> FuncSpecBackendResponseType {
        self.response_type
    }

    pub fn is_transformation(&self) -> bool {
        self.is_transformation
    }

    pub fn last_updated_at(&self) -> Option<DateTime<Utc>> {
        self.last_updated_at
    }

    pub fn hidden(&self) -> bool {
        self.hidden
    }

    pub fn link(&self) -> Option<&Url> {
        self.link.as_ref()
    }
}

#[derive(Clone, Debug)]
pub struct SiPkgFunc<'a> {
    name: String,
    data: Option<SiPkgFuncData>,
    unique_id: String,
    deleted: bool,
    is_from_builtin: Option<bool>,

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
                ));
            }
        };

        Ok(Self {
            name: func_node.name,
            data: func_node.data.map(|data| SiPkgFuncData {
                name: data.name,
                display_name: data.display_name,
                description: data.description,
                handler: data.handler,
                code_base64: data.code_base64,
                backend_kind: data.backend_kind,
                response_type: data.response_type,
                hidden: data.hidden,
                link: data.link,
                is_transformation: data.is_transformation,
                last_updated_at: data.last_updated_at,
            }),
            hash: func_hashed_node.hash(),
            unique_id: func_node.unique_id,
            deleted: func_node.deleted,
            is_from_builtin: func_node.is_from_builtin,
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

    pub fn data(&self) -> Option<&SiPkgFuncData> {
        self.data.as_ref()
    }

    pub fn deleted(&self) -> bool {
        self.deleted
    }

    pub fn display_name(&self) -> Option<&str> {
        match self.data() {
            None => None,
            Some(data) => data.display_name.as_deref(),
        }
    }

    pub fn description(&self) -> Option<&str> {
        match self.data() {
            None => None,
            Some(data) => data.description.as_deref(),
        }
    }

    pub fn handler(&self) -> Option<&str> {
        self.data().map(|data| data.handler.as_str())
    }

    pub fn code_base64(&self) -> Option<&str> {
        self.data().map(|data| data.code_base64.as_str())
    }

    pub fn backend_kind(&self) -> Option<FuncSpecBackendKind> {
        self.data().map(|data| data.backend_kind)
    }

    pub fn response_type(&self) -> Option<FuncSpecBackendResponseType> {
        self.data().map(|data| data.response_type)
    }

    pub fn hidden(&self) -> Option<bool> {
        self.data().map(|data| data.hidden)
    }

    pub fn link(&self) -> Option<&Url> {
        match self.data() {
            None => None,
            Some(data) => data.link.as_ref(),
        }
    }

    pub fn is_from_builtin(&self) -> Option<bool> {
        self.is_from_builtin
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }

    pub fn unique_id(&self) -> &str {
        self.unique_id.as_str()
    }

    pub fn source(&self) -> &Source<'a> {
        &self.source
    }
}

impl<'a> TryFrom<SiPkgFunc<'a>> for FuncSpec {
    type Error = SiPkgError;

    fn try_from(value: SiPkgFunc<'a>) -> Result<Self, Self::Error> {
        let mut builder = FuncSpec::builder();
        let mut data_builder = FuncSpecData::builder();

        builder
            .name(&value.name)
            .unique_id(&value.unique_id)
            .deleted(value.deleted)
            .is_from_builtin(value.is_from_builtin);

        if let Some(data) = value.data() {
            data_builder
                .name(&data.name)
                .handler(&data.handler)
                .code_base64(&data.code_base64)
                .backend_kind(data.backend_kind)
                .response_type(data.response_type)
                .hidden(data.hidden);

            if let Some(display_name) = &data.display_name {
                data_builder.display_name(display_name);
            }

            if let Some(description) = &data.description {
                data_builder.description(description);
            }

            if let Some(link) = &data.link {
                data_builder.link(link.to_owned());
            }

            data_builder.is_transformation(data.is_transformation);
            data_builder.last_updated_at(data.last_updated_at);

            builder.data(data_builder.build()?);
        }

        for argument in value.arguments()? {
            builder.argument(argument.try_into()?);
        }

        Ok(builder.build()?)
    }
}
