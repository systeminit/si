use std::collections::HashMap;

use object_tree::{
    Hash,
    HashedNode,
};
use petgraph::prelude::*;
use url::Url;

use super::{
    PkgResult,
    SiPkgAttrFuncInput,
    SiPkgError,
    SiPkgMapKeyFunc,
    Source,
};
use crate::{
    PropSpecWidgetKind,
    node::{
        PkgNode,
        PropChildNode,
        PropNode,
        PropNodeData,
    },
};

#[derive(Clone, Debug)]
pub struct SiPkgPropData {
    pub name: String,
    pub default_value: Option<serde_json::Value>,
    pub func_unique_id: Option<String>,
    pub widget_kind: PropSpecWidgetKind,
    pub widget_options: Option<serde_json::Value>,
    pub doc_link: Option<Url>,
    pub hidden: bool,
    pub documentation: Option<String>,
    pub validation_format: Option<String>,
    pub ui_optionals: HashMap<String, serde_json::Value>,
}

#[remain::sorted]
#[derive(Clone, Debug)]
pub enum SiPkgProp<'a> {
    Array {
        name: String,
        data: Option<SiPkgPropData>,
        unique_id: Option<String>,
        hash: Hash,
        source: Source<'a>,
    },
    Boolean {
        name: String,
        data: Option<SiPkgPropData>,
        unique_id: Option<String>,
        hash: Hash,
        source: Source<'a>,
    },
    Float {
        name: String,
        data: Option<SiPkgPropData>,
        unique_id: Option<String>,
        hash: Hash,
        source: Source<'a>,
    },
    Json {
        name: String,
        data: Option<SiPkgPropData>,
        unique_id: Option<String>,
        hash: Hash,
        source: Source<'a>,
    },
    Map {
        name: String,
        data: Option<SiPkgPropData>,
        unique_id: Option<String>,
        hash: Hash,
        source: Source<'a>,
    },
    Number {
        name: String,
        data: Option<SiPkgPropData>,
        unique_id: Option<String>,
        hash: Hash,
        source: Source<'a>,
    },
    Object {
        name: String,
        data: Option<SiPkgPropData>,
        unique_id: Option<String>,
        hash: Hash,
        source: Source<'a>,
        child_order: Option<Vec<String>>,
    },
    String {
        name: String,
        data: Option<SiPkgPropData>,
        unique_id: Option<String>,
        hash: Hash,
        source: Source<'a>,
    },
}

macro_rules! impl_prop_child_from_graph {
    ($fn_name:ident, PropChildNode::$child_node:ident, $pkg_type:ident) => {
        pub fn $fn_name(&self) -> PkgResult<Vec<$pkg_type>> {
            Ok(match self {
                SiPkgProp::Map { source, .. }
                | SiPkgProp::Array { source, .. }
                | SiPkgProp::Json { source, .. }
                | SiPkgProp::String { source, .. }
                | SiPkgProp::Number { source, .. }
                | SiPkgProp::Float { source, .. }
                | SiPkgProp::Object { source, .. }
                | SiPkgProp::Boolean { source, .. } => {
                    let mut entries = vec![];
                    if let Some(child_idxs) = source
                        .graph
                        .neighbors_directed(source.node_idx, Outgoing)
                        .find(|node_idx| {
                            matches!(
                                &source.graph[*node_idx].inner(),
                                PkgNode::PropChild(PropChildNode::$child_node)
                            )
                        })
                    {
                        let child_node_idxs: Vec<_> = source
                            .graph
                            .neighbors_directed(child_idxs, Outgoing)
                            .collect();

                        for child_idx in child_node_idxs {
                            entries.push($pkg_type::from_graph(source.graph, child_idx)?);
                        }
                    }

                    entries
                }
            })
        }
    };
}

impl<'a> SiPkgProp<'a> {
    impl_prop_child_from_graph!(inputs, PropChildNode::AttrFuncInputs, SiPkgAttrFuncInput);
    impl_prop_child_from_graph!(map_key_funcs, PropChildNode::MapKeyFuncs, SiPkgMapKeyFunc);

    pub fn from_graph(
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
                ));
            }
        };

        let hash = prop_hashed_node.hash();
        let source = Source::new(graph, node_idx);

        let (name, data, unique_id) = match &prop_node {
            PropNode::Array {
                name,
                data,
                unique_id,
            }
            | PropNode::Json {
                name,
                data,
                unique_id,
            }
            | PropNode::Boolean {
                name,
                data,
                unique_id,
            }
            | PropNode::Map {
                name,
                data,
                unique_id,
            }
            | PropNode::Float {
                name,
                data,
                unique_id,
            }
            | PropNode::Integer {
                name,
                data,
                unique_id,
            }
            | PropNode::Object {
                name,
                data,
                unique_id,
                ..
            }
            | PropNode::String {
                name,
                data,
                unique_id,
            } => (
                name.to_owned(),
                data.to_owned().map(
                    |PropNodeData {
                         name,
                         default_value,
                         func_unique_id,
                         widget_kind,
                         widget_options,
                         hidden,
                         doc_link,
                         documentation,
                         validation_format,
                         ui_optionals,
                     }| SiPkgPropData {
                        name,
                        default_value,
                        func_unique_id,
                        widget_kind,
                        widget_options,
                        hidden,
                        doc_link,
                        documentation,
                        validation_format,
                        ui_optionals,
                    },
                ),
                unique_id.to_owned(),
            ),
        };

        Ok(match prop_node {
            PropNode::String { .. } => Self::String {
                name,
                data,
                unique_id,

                hash,
                source,
            },
            PropNode::Integer { .. } => Self::Number {
                name,
                data,
                unique_id,

                hash,
                source,
            },
            PropNode::Float { .. } => Self::Float {
                name,
                data,
                unique_id,

                hash,
                source,
            },
            PropNode::Json { .. } => Self::Json {
                name,
                data,
                unique_id,

                hash,
                source,
            },
            PropNode::Boolean { .. } => Self::Boolean {
                name,
                data,
                unique_id,

                hash,
                source,
            },
            PropNode::Map { .. } => Self::Map {
                name,
                data,
                unique_id,

                hash,
                source,
            },
            PropNode::Array { .. } => Self::Array {
                name,
                data,
                unique_id,

                hash,
                source,
            },
            PropNode::Object { child_order, .. } => Self::Object {
                name,
                data,
                unique_id,

                hash,
                source,
                child_order,
            },
        })
    }

    pub fn data(&self) -> Option<&SiPkgPropData> {
        match self {
            SiPkgProp::Array { data, .. }
            | SiPkgProp::Boolean { data, .. }
            | SiPkgProp::Json { data, .. }
            | SiPkgProp::Map { data, .. }
            | SiPkgProp::Number { data, .. }
            | SiPkgProp::Float { data, .. }
            | SiPkgProp::Object { data, .. }
            | SiPkgProp::String { data, .. } => data.as_ref(),
        }
    }

    pub fn unique_id(&self) -> Option<&str> {
        match self {
            SiPkgProp::Array { unique_id, .. }
            | SiPkgProp::Boolean { unique_id, .. }
            | SiPkgProp::Json { unique_id, .. }
            | SiPkgProp::Map { unique_id, .. }
            | SiPkgProp::Float { unique_id, .. }
            | SiPkgProp::Number { unique_id, .. }
            | SiPkgProp::Object { unique_id, .. }
            | SiPkgProp::String { unique_id, .. } => unique_id.as_deref(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::String { name, .. }
            | Self::Number { name, .. }
            | Self::Float { name, .. }
            | Self::Json { name, .. }
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
            | Self::Float { hash, .. }
            | Self::Json { hash, .. }
            | Self::Boolean { hash, .. }
            | Self::Map { hash, .. }
            | Self::Array { hash, .. }
            | Self::Object { hash, .. } => *hash,
        }
    }

    pub fn source(&self) -> &Source<'a> {
        match self {
            Self::String { source, .. }
            | Self::Json { source, .. }
            | Self::Number { source, .. }
            | Self::Float { source, .. }
            | Self::Boolean { source, .. }
            | Self::Map { source, .. }
            | Self::Array { source, .. }
            | Self::Object { source, .. } => source,
        }
    }

    pub fn child_order(&self) -> Option<&Vec<String>> {
        match self {
            Self::Object { child_order, .. } => child_order.as_ref(),
            _ => None,
        }
    }
}
