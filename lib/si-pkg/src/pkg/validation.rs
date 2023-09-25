use object_tree::{Hash, HashedNode};
use petgraph::prelude::*;

use super::{PkgResult, SiPkgError, Source};

use crate::{node::PkgNode, ValidationSpec, ValidationSpecKind};

#[remain::sorted]
#[derive(Clone, Debug)]
pub enum SiPkgValidation<'a> {
    CustomValidation {
        func_unique_id: String,
        unique_id: Option<String>,
        deleted: bool,

        hash: Hash,
        source: Source<'a>,
    },
    IntegerIsBetweenTwoIntegers {
        lower_bound: i64,
        upper_bound: i64,
        unique_id: Option<String>,
        deleted: bool,

        hash: Hash,
        source: Source<'a>,
    },
    IntegerIsNotEmpty {
        unique_id: Option<String>,
        deleted: bool,

        hash: Hash,
        source: Source<'a>,
    },
    StringEquals {
        expected: String,
        unique_id: Option<String>,
        deleted: bool,

        hash: Hash,
        source: Source<'a>,
    },
    StringHasPrefix {
        expected: String,
        unique_id: Option<String>,
        deleted: bool,

        hash: Hash,
        source: Source<'a>,
    },
    StringInStringArray {
        expected: Vec<String>,
        display_expected: bool,
        unique_id: Option<String>,
        deleted: bool,

        hash: Hash,
        source: Source<'a>,
    },
    StringIsHexColor {
        unique_id: Option<String>,
        deleted: bool,

        hash: Hash,
        source: Source<'a>,
    },
    StringIsNotEmpty {
        unique_id: Option<String>,
        deleted: bool,

        hash: Hash,
        source: Source<'a>,
    },
    StringIsValidIpAddr {
        unique_id: Option<String>,
        deleted: bool,

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
                    PkgNode::VALIDATION_KIND_STR,
                    unexpected.node_kind_str(),
                ))
            }
        };

        let unique_id = node.unique_id;
        let deleted = node.deleted;
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
                    unique_id,
                    deleted,
                    hash,
                    source,
                }
            }
            ValidationSpecKind::IntegerIsNotEmpty => SiPkgValidation::IntegerIsNotEmpty {
                unique_id,
                deleted,
                hash,
                source,
            },
            ValidationSpecKind::StringEquals => SiPkgValidation::StringEquals {
                expected: node
                    .expected_string
                    .ok_or(SiPkgError::ValidationMissingField(
                        "expected_string".to_string(),
                    ))?,
                unique_id,
                deleted,
                hash,
                source,
            },
            ValidationSpecKind::StringHasPrefix => SiPkgValidation::StringHasPrefix {
                expected: node
                    .expected_string
                    .ok_or(SiPkgError::ValidationMissingField(
                        "expected_string".to_string(),
                    ))?,
                unique_id,
                deleted,
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
                    unique_id,
                    deleted,
                    hash,
                    source,
                }
            }

            ValidationSpecKind::StringIsValidIpAddr => SiPkgValidation::StringIsValidIpAddr {
                unique_id,
                deleted,
                hash,
                source,
            },
            ValidationSpecKind::StringIsHexColor => SiPkgValidation::StringIsHexColor {
                unique_id,
                deleted,
                hash,
                source,
            },
            ValidationSpecKind::StringIsNotEmpty => SiPkgValidation::StringIsNotEmpty {
                unique_id,
                deleted,
                hash,
                source,
            },
            ValidationSpecKind::CustomValidation => {
                SiPkgValidation::CustomValidation {
                    func_unique_id: node.func_unique_id.ok_or(
                        SiPkgError::ValidationMissingField("func_unique_id".to_string()),
                    )?,
                    unique_id,
                    deleted,
                    hash,
                    source,
                }
            }
        })
    }
}

impl<'a> TryFrom<SiPkgValidation<'a>> for ValidationSpec {
    type Error = SiPkgError;

    fn try_from(value: SiPkgValidation<'a>) -> Result<Self, Self::Error> {
        let mut builder = ValidationSpec::builder();

        let (unique_id, deleted) = match &value {
            SiPkgValidation::CustomValidation {
                unique_id, deleted, ..
            }
            | SiPkgValidation::IntegerIsBetweenTwoIntegers {
                unique_id, deleted, ..
            }
            | SiPkgValidation::IntegerIsNotEmpty {
                unique_id, deleted, ..
            }
            | SiPkgValidation::StringEquals {
                unique_id, deleted, ..
            }
            | SiPkgValidation::StringHasPrefix {
                unique_id, deleted, ..
            }
            | SiPkgValidation::StringInStringArray {
                unique_id, deleted, ..
            }
            | SiPkgValidation::StringIsHexColor {
                unique_id, deleted, ..
            }
            | SiPkgValidation::StringIsNotEmpty {
                unique_id, deleted, ..
            }
            | SiPkgValidation::StringIsValidIpAddr {
                unique_id, deleted, ..
            } => (unique_id.to_owned(), *deleted),
        };

        if let Some(unique_id) = unique_id {
            builder.unique_id(unique_id);
        }
        builder.deleted(deleted);

        match value {
            SiPkgValidation::IntegerIsBetweenTwoIntegers {
                lower_bound,
                upper_bound,
                ..
            } => {
                builder.kind(ValidationSpecKind::IntegerIsBetweenTwoIntegers);
                builder.lower_bound(lower_bound);
                builder.upper_bound(upper_bound);
            }
            SiPkgValidation::IntegerIsNotEmpty { .. } => {
                builder.kind(ValidationSpecKind::IntegerIsNotEmpty);
            }
            SiPkgValidation::StringEquals { expected, .. } => {
                builder.kind(ValidationSpecKind::StringEquals);
                builder.expected_string(expected);
            }
            SiPkgValidation::StringHasPrefix { expected, .. } => {
                builder.kind(ValidationSpecKind::StringHasPrefix);
                builder.expected_string(expected);
            }
            SiPkgValidation::CustomValidation { func_unique_id, .. } => {
                builder.kind(ValidationSpecKind::CustomValidation);
                builder.func_unique_id(&func_unique_id);
            }
            SiPkgValidation::StringInStringArray {
                expected,
                display_expected,
                ..
            } => {
                builder.kind(ValidationSpecKind::StringInStringArray);
                builder.expected_string_array(expected);
                builder.display_expected(display_expected);
            }
            SiPkgValidation::StringIsValidIpAddr { .. } => {
                builder.kind(ValidationSpecKind::StringIsValidIpAddr);
            }
            SiPkgValidation::StringIsHexColor { .. } => {
                builder.kind(ValidationSpecKind::StringIsHexColor);
            }
            SiPkgValidation::StringIsNotEmpty { .. } => {
                builder.kind(ValidationSpecKind::StringIsNotEmpty);
            }
        }

        Ok(builder.build()?)
    }
}
