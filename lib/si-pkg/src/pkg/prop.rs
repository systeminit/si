use object_tree::{Hash, HashedNode};
use petgraph::prelude::*;

use super::{PkgResult, SiPkgAttrFuncInput, SiPkgError, SiPkgValidation, Source};

use crate::{
    node::{PkgNode, PropChildNode, PropNode},
    FuncUniqueId, PropSpecSharedInfo,
};

#[derive(Clone, Debug)]
pub enum SiPkgProp<'a> {
    String {
        default_value: Option<String>,
        info: PropSpecSharedInfo,
        hash: Hash,
        source: Source<'a>,
    },
    Number {
        default_value: Option<i64>,
        info: PropSpecSharedInfo,
        hash: Hash,
        source: Source<'a>,
    },
    Boolean {
        default_value: Option<bool>,
        info: PropSpecSharedInfo,
        hash: Hash,
        source: Source<'a>,
    },
    Map {
        default_value: Option<serde_json::Value>,
        info: PropSpecSharedInfo,
        hash: Hash,
        source: Source<'a>,
    },
    Array {
        default_value: Option<serde_json::Value>,
        info: PropSpecSharedInfo,
        hash: Hash,
        source: Source<'a>,
    },
    Object {
        default_value: Option<serde_json::Value>,
        info: PropSpecSharedInfo,
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
                | SiPkgProp::String { source, .. }
                | SiPkgProp::Number { source, .. }
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
    impl_prop_child_from_graph!(validations, PropChildNode::Validations, SiPkgValidation);
    impl_prop_child_from_graph!(inputs, PropChildNode::AttrFuncInputs, SiPkgAttrFuncInput);

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
                ))
            }
        };

        let hash = prop_hashed_node.hash();
        let source = Source::new(graph, node_idx);

        Ok(match prop_node {
            PropNode::String {
                default_value,
                info,
            } => Self::String {
                default_value,
                info,
                hash,
                source,
            },
            PropNode::Integer {
                default_value,
                info,
            } => Self::Number {
                default_value,
                info,
                hash,
                source,
            },
            PropNode::Boolean {
                default_value,
                info,
            } => Self::Boolean {
                default_value,
                info,
                hash,
                source,
            },
            PropNode::Map {
                default_value,
                info,
            } => Self::Map {
                default_value,
                info,
                hash,
                source,
            },
            PropNode::Array {
                default_value,
                info,
            } => Self::Array {
                default_value,
                info,
                hash,
                source,
            },
            PropNode::Object {
                default_value,
                info,
            } => Self::Object {
                default_value,
                info,
                hash,
                source,
            },
        })
    }

    pub fn info(&self) -> &PropSpecSharedInfo {
        match self {
            Self::String { info, .. }
            | Self::Number { info, .. }
            | Self::Boolean { info, .. }
            | Self::Map { info, .. }
            | Self::Array { info, .. }
            | Self::Object { info, .. } => &info,
        }
    }

    pub fn func_unique_id(&self) -> Option<FuncUniqueId> {
        self.info().func_unique_id
    }

    pub fn name(&self) -> &str {
        &self.info().name
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

    pub fn source(&self) -> &Source<'a> {
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
