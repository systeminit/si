use object_tree::{Hash, HashedNode};
use petgraph::prelude::*;

use super::{PkgResult, SiPkgAttrFuncInput, SiPkgError, Source};

use crate::{
    node::{AttributeValueChildNode, PkgNode},
    AttrFuncInputSpec, AttributeValuePath, AttributeValueSpec, FuncSpecBackendKind,
    FuncSpecBackendResponseType,
};

pub struct SiPkgAttributeValue<'a> {
    parent_path: Option<AttributeValuePath>,
    path: AttributeValuePath,
    func_unique_id: String,
    func_binding_args: serde_json::Value,
    handler: Option<String>,
    backend_kind: FuncSpecBackendKind,
    response_type: FuncSpecBackendResponseType,
    code_base64: Option<String>,
    unprocessed_value: Option<serde_json::Value>,
    value: Option<serde_json::Value>,
    output_stream: Option<serde_json::Value>,
    is_proxy: bool,
    sealed_proxy: bool,
    component_specific: bool,

    hash: Hash,
    source: Source<'a>,
}

macro_rules! impl_attribute_value_children_from_graph {
    ($fn_name:ident, AttributeValueChildNode::$child_node:ident, $pkg_type:ident) => {
        pub fn $fn_name(&self) -> PkgResult<Vec<$pkg_type>> {
            let mut entries = vec![];
            if let Some(child_idxs) = self
                .source
                .graph
                .neighbors_directed(self.source.node_idx, Outgoing)
                .find(|node_idx| {
                    matches!(
                        &self.source.graph[*node_idx].inner(),
                        PkgNode::AttributeValueChild(AttributeValueChildNode::$child_node)
                    )
                })
            {
                let child_node_idxs: Vec<_> = self
                    .source
                    .graph
                    .neighbors_directed(child_idxs, Outgoing)
                    .collect();

                for child_idx in child_node_idxs {
                    entries.push($pkg_type::from_graph(self.source.graph, child_idx)?);
                }
            }

            Ok(entries)
        }
    };
}

impl<'a> SiPkgAttributeValue<'a> {
    pub fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
        let hashed_node = &graph[node_idx];
        let node = match hashed_node.inner() {
            PkgNode::AttributeValue(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::ATTRIBUTE_VALUE_KIND_STR,
                    unexpected.node_kind_str(),
                ))
            }
        };

        Ok(Self {
            parent_path: node.parent_path,
            path: node.path,
            func_unique_id: node.func_unique_id,
            func_binding_args: node.func_binding_args,
            handler: node.handler,
            backend_kind: node.backend_kind,
            response_type: node.response_type,
            code_base64: node.code_base64,
            unprocessed_value: node.unprocessed_value,
            value: node.value,
            output_stream: node.output_stream,
            is_proxy: node.is_proxy,
            sealed_proxy: node.sealed_proxy,
            component_specific: node.component_specific,

            hash: hashed_node.hash(),
            source: Source::new(graph, node_idx),
        })
    }

    pub fn parent_path(&self) -> Option<&AttributeValuePath> {
        self.parent_path.as_ref()
    }

    pub fn path(&self) -> &AttributeValuePath {
        &self.path
    }

    pub fn func_unique_id(&self) -> &str {
        self.func_unique_id.as_str()
    }

    pub fn func_binding_args(&self) -> &serde_json::Value {
        &self.func_binding_args
    }

    pub fn handler(&self) -> Option<&str> {
        self.handler.as_deref()
    }

    pub fn backend_kind(&self) -> FuncSpecBackendKind {
        self.backend_kind
    }

    pub fn response_type(&self) -> FuncSpecBackendResponseType {
        self.response_type
    }

    pub fn code_base64(&self) -> Option<&str> {
        self.code_base64.as_deref()
    }

    pub fn unprocessed_value(&self) -> Option<&serde_json::Value> {
        self.unprocessed_value.as_ref()
    }

    pub fn value(&self) -> Option<&serde_json::Value> {
        self.value.as_ref()
    }

    pub fn output_stream(&self) -> Option<&serde_json::Value> {
        self.output_stream.as_ref()
    }

    pub fn is_proxy(&self) -> bool {
        self.is_proxy
    }

    pub fn sealed_proxy(&self) -> bool {
        self.sealed_proxy
    }

    pub fn component_specific(&self) -> bool {
        self.component_specific
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }

    pub fn source(&self) -> &Source<'a> {
        &self.source
    }

    impl_attribute_value_children_from_graph!(
        inputs,
        AttributeValueChildNode::AttrFuncInputs,
        SiPkgAttrFuncInput
    );
}

impl<'a> TryFrom<SiPkgAttributeValue<'a>> for AttributeValueSpec {
    type Error = SiPkgError;

    fn try_from(value: SiPkgAttributeValue<'a>) -> Result<Self, Self::Error> {
        let mut builder = AttributeValueSpec::builder();

        if let Some(parent_path) = value.parent_path() {
            builder.parent_path(parent_path.to_owned());
        }

        if let Some(handler) = value.handler() {
            builder.handler(handler);
        }

        if let Some(code_base64) = value.code_base64() {
            builder.code_base64(code_base64);
        }

        if let Some(unprocessed_value) = value.unprocessed_value() {
            builder.unprocessed_value(unprocessed_value.to_owned());
        }

        if let Some(value) = value.value() {
            builder.value(value.to_owned());
        }

        if let Some(output_stream) = value.output_stream() {
            builder.output_stream(output_stream.to_owned());
        }

        builder
            .path(value.path().to_owned())
            .func_unique_id(value.func_unique_id())
            .func_binding_args(value.func_binding_args().to_owned())
            .backend_kind(value.backend_kind())
            .response_type(value.response_type())
            .sealed_proxy(value.sealed_proxy())
            .component_specific(value.component_specific());

        for input in value.inputs()? {
            builder.input(AttrFuncInputSpec::try_from(input)?);
        }

        Ok(builder.build()?)
    }
}
