use std::{
    io::{BufRead, Write},
    str::FromStr,
};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NodeChild, NodeKind, NodeWithChildren,
    ReadBytes, WriteBytes,
};

use crate::{FuncUniqueId, ValidationSpec, ValidationSpecKind};

use super::PkgNode;

const KEY_KIND_STR: &str = "kind";
const KEY_UPPER_BOUND_STR: &str = "upper_bound";
const KEY_LOWER_BOUND_STR: &str = "lower_bound";
const KEY_EXPECTED_STRING_STR: &str = "expected_string";
const KEY_EXPECTED_STRING_ARRAY_STR: &str = "expected_string_array";
const KEY_DISPLAY_EXPECTED_STR: &str = "display_expected";
const KEY_FUNC_UNIQUE_ID_STR: &str = "func_unique_id";

#[derive(Clone, Debug)]
pub struct ValidationNode {
    pub kind: ValidationSpecKind,
    pub upper_bound: Option<i64>,
    pub lower_bound: Option<i64>,
    pub expected_string: Option<String>,
    pub expected_string_array: Option<Vec<String>>,
    pub display_expected: Option<bool>,
    pub func_unique_id: Option<FuncUniqueId>,
}

impl Default for ValidationNode {
    fn default() -> Self {
        Self {
            kind: ValidationSpecKind::CustomValidation,
            upper_bound: None,
            lower_bound: None,
            expected_string: None,
            expected_string_array: None,
            display_expected: None,
            func_unique_id: None,
        }
    }
}

impl WriteBytes for ValidationNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_KIND_STR, self.kind)?;

        match self.kind {
            ValidationSpecKind::IntegerIsBetweenTwoIntegers => {
                write_key_value_line(
                    writer,
                    KEY_UPPER_BOUND_STR,
                    self.upper_bound
                        .map(|i| i.to_string())
                        .unwrap_or("".to_string()),
                )?;
                write_key_value_line(
                    writer,
                    KEY_LOWER_BOUND_STR,
                    self.lower_bound
                        .map(|i| i.to_string())
                        .unwrap_or("".to_string()),
                )?;
            }
            ValidationSpecKind::StringEquals | ValidationSpecKind::StringHasPrefix => {
                write_key_value_line(
                    writer,
                    KEY_EXPECTED_STRING_STR,
                    self.expected_string.clone().unwrap_or("".to_string()),
                )?
            }
            ValidationSpecKind::StringInStringArray => {
                write_key_value_line(
                    writer,
                    KEY_EXPECTED_STRING_ARRAY_STR,
                    serde_json::to_string(&self.expected_string_array.clone().unwrap_or(vec![]))
                        .map_err(GraphError::parse)?,
                )?;
                write_key_value_line(
                    writer,
                    KEY_DISPLAY_EXPECTED_STR,
                    self.display_expected
                        .map(|display_expected| display_expected.to_string())
                        .unwrap_or("".to_string()),
                )?
            }
            ValidationSpecKind::CustomValidation => write_key_value_line(
                writer,
                KEY_FUNC_UNIQUE_ID_STR,
                self.func_unique_id
                    .map(|id| id.to_string())
                    .unwrap_or("".to_string()),
            )?,
            ValidationSpecKind::StringIsValidIpAddr
            | ValidationSpecKind::StringIsHexColor
            | ValidationSpecKind::StringIsNotEmpty => {}
        }

        Ok(())
    }
}

impl ReadBytes for ValidationNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Self, GraphError>
    where
        Self: std::marker::Sized,
    {
        let kind_str = read_key_value_line(reader, KEY_KIND_STR)?;
        let kind = ValidationSpecKind::from_str(&kind_str).map_err(GraphError::parse)?;
        let mut upper_bound = None;
        let mut lower_bound = None;
        let mut expected_string = None;
        let mut expected_string_array = None;
        let mut display_expected = None;
        let mut func_unique_id = None;

        match kind {
            ValidationSpecKind::IntegerIsBetweenTwoIntegers => {
                let upper_bound_str = read_key_value_line(reader, KEY_UPPER_BOUND_STR)?;
                upper_bound = Some(i64::from_str(&upper_bound_str).map_err(GraphError::parse)?);

                let lower_bound_str = read_key_value_line(reader, KEY_LOWER_BOUND_STR)?;
                lower_bound = Some(i64::from_str(&lower_bound_str).map_err(GraphError::parse)?);
            }
            ValidationSpecKind::StringEquals | ValidationSpecKind::StringHasPrefix => {
                let expected_string_str = read_key_value_line(reader, KEY_EXPECTED_STRING_STR)?;
                if !expected_string_str.is_empty() {
                    expected_string = Some(expected_string_str);
                }
            }
            ValidationSpecKind::StringInStringArray => {
                let expected_string_array_str =
                    read_key_value_line(reader, KEY_EXPECTED_STRING_ARRAY_STR)?;
                let expected_string_array_json: Vec<String> =
                    serde_json::from_str(&expected_string_array_str).map_err(GraphError::parse)?;
                if !expected_string_array_json.is_empty() {
                    expected_string_array = Some(expected_string_array_json);
                }

                let display_expected_str = read_key_value_line(reader, KEY_DISPLAY_EXPECTED_STR)?;
                if !display_expected_str.is_empty() {
                    display_expected =
                        Some(bool::from_str(&display_expected_str).map_err(GraphError::parse)?);
                }
            }
            ValidationSpecKind::CustomValidation => {
                let func_unique_id_str = read_key_value_line(reader, KEY_FUNC_UNIQUE_ID_STR)?;
                func_unique_id =
                    Some(FuncUniqueId::from_str(&func_unique_id_str).map_err(GraphError::parse)?);
            }
            ValidationSpecKind::StringIsValidIpAddr
            | ValidationSpecKind::StringIsHexColor
            | ValidationSpecKind::StringIsNotEmpty => {}
        }

        Ok(Self {
            kind,
            lower_bound,
            upper_bound,
            expected_string,
            expected_string_array,
            display_expected,
            func_unique_id,
        })
    }
}

impl NodeChild for ValidationSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Leaf,
            Self::NodeType::Validation(match self {
                ValidationSpec::IntegerIsBetweenTwoIntegers {
                    lower_bound,
                    upper_bound,
                } => ValidationNode {
                    kind: ValidationSpecKind::IntegerIsBetweenTwoIntegers,
                    upper_bound: Some(*upper_bound),
                    lower_bound: Some(*lower_bound),
                    ..ValidationNode::default()
                },
                ValidationSpec::StringEquals { expected } => ValidationNode {
                    kind: ValidationSpecKind::StringEquals,
                    expected_string: Some(expected.clone()),
                    ..ValidationNode::default()
                },
                ValidationSpec::StringHasPrefix { expected } => ValidationNode {
                    kind: ValidationSpecKind::StringHasPrefix,
                    expected_string: Some(expected.clone()),
                    ..ValidationNode::default()
                },
                ValidationSpec::StringInStringArray {
                    expected,
                    display_expected,
                } => ValidationNode {
                    kind: ValidationSpecKind::StringInStringArray,
                    expected_string_array: Some(expected.clone()),
                    display_expected: Some(*display_expected),
                    ..ValidationNode::default()
                },
                ValidationSpec::StringIsValidIpAddr => ValidationNode {
                    kind: ValidationSpecKind::StringIsValidIpAddr,
                    ..ValidationNode::default()
                },
                ValidationSpec::StringIsHexColor => ValidationNode {
                    kind: ValidationSpecKind::StringIsHexColor,
                    ..ValidationNode::default()
                },
                ValidationSpec::StringIsNotEmpty => ValidationNode {
                    kind: ValidationSpecKind::StringIsNotEmpty,
                    ..ValidationNode::default()
                },
                ValidationSpec::CustomValidation { func_unique_id } => ValidationNode {
                    kind: ValidationSpecKind::CustomValidation,
                    func_unique_id: Some(*func_unique_id),
                    ..ValidationNode::default()
                },
            }),
            vec![],
        )
    }
}
