use object_tree::{Hash, HashedNode};
use petgraph::prelude::*;

use super::{PkgResult, SiPkgError, Source};

use crate::{node::PkgNode, spec::ValidationSpecKind};

#[derive(Clone, Debug)]
pub enum SiPkgValidation<'a> {
    IntegerIsBetweenTwoIntegers {
        lower_bound: i64,
        upper_bound: i64,
        hash: Hash,
        source: Source<'a>,
    },
    StringEquals {
        expected: String,
        hash: Hash,
        source: Source<'a>,
    },
    StringHasPrefix {
        expected: String,
        hash: Hash,
        source: Source<'a>,
    },
    StringInStringArray {
        expected: Vec<String>,
        display_expected: bool,
        hash: Hash,
        source: Source<'a>,
    },
    StringIsValidIpAddr {
        hash: Hash,
        source: Source<'a>,
    },
    StringIsHexColor {
        hash: Hash,
        source: Source<'a>,
    },
    StringIsNotEmpty {
        hash: Hash,
        source: Source<'a>,
    },
    CustomValidation {
        func_unique_id: Hash,
        hash: Hash,
        source: Source<'a>,
    },
}

impl<'a> SiPkgValidation<'a> {
    pub fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
        let hashed_node = &graph[node_idx];
        let node = match hashed_node.inner() {
            PkgNode::Validation(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::PROP_KIND_STR,
                    unexpected.node_kind_str(),
                ))
            }
        };

        let hash = hashed_node.hash();
        let source = Source::new(graph, node_idx);

        Ok(match node.kind {
            ValidationSpecKind::IntegerIsBetweenTwoIntegers => {
                SiPkgValidation::IntegerIsBetweenTwoIntegers {
                    upper_bound: node.upper_bound.ok_or(SiPkgError::ValidationMissingField(
                        "upper_bound".to_string(),
                    ))?,
                    lower_bound: node.lower_bound.ok_or(SiPkgError::ValidationMissingField(
                        "upper_bound".to_string(),
                    ))?,
                    hash,
                    source,
                }
            }
            ValidationSpecKind::StringEquals => SiPkgValidation::StringEquals {
                expected: node
                    .expected_string
                    .ok_or(SiPkgError::ValidationMissingField(
                        "expected_string".to_string(),
                    ))?,
                hash,
                source,
            },
            ValidationSpecKind::StringHasPrefix => SiPkgValidation::StringHasPrefix {
                expected: node
                    .expected_string
                    .ok_or(SiPkgError::ValidationMissingField(
                        "expected_string".to_string(),
                    ))?,
                hash,
                source,
            },
            ValidationSpecKind::StringInStringArray => {
                SiPkgValidation::StringInStringArray {
                    expected: node.expected_string_array.ok_or(
                        SiPkgError::ValidationMissingField("expected_string_array".to_string()),
                    )?,
                    display_expected: node.display_expected.ok_or(
                        SiPkgError::ValidationMissingField("display_expected".to_string()),
                    )?,
                    hash,
                    source,
                }
            }

            ValidationSpecKind::StringIsValidIpAddr => {
                SiPkgValidation::StringIsValidIpAddr { hash, source }
            }
            ValidationSpecKind::StringIsHexColor => {
                SiPkgValidation::StringIsHexColor { hash, source }
            }
            ValidationSpecKind::StringIsNotEmpty => {
                SiPkgValidation::StringIsNotEmpty { hash, source }
            }
            ValidationSpecKind::CustomValidation => {
                SiPkgValidation::CustomValidation {
                    func_unique_id: node.func_unique_id.ok_or(
                        SiPkgError::ValidationMissingField("func_unique_id".to_string()),
                    )?,
                    hash,
                    source,
                }
            }
        })
    }
}
